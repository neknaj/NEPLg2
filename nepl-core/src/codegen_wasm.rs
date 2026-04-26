//! WASM backend for NEPLG2.

#![no_std]
extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

use alloc::borrow::Cow;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ElementMode, ElementSection, ElementSegment, Elements,
    EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    MemArg, MemorySection, MemoryType, Module, RefType, TableSection, TableType, TypeSection,
    ValType,
};

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::*;
use crate::runtime_helpers::{self, RuntimeHelperKind};
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

macro_rules! wasm_log {
    ($($arg:tt)*) => {{
        #[cfg(target_os = "none")]
        {
            let _ = core::format_args!($($arg)*);
        }
        #[cfg(not(target_os = "none"))]
        {
            std::eprintln!($($arg)*);
        }
    }};
}

#[derive(Debug)]
pub struct CodegenResult {
    pub bytes: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}

type LowerResult<T> = Result<T, Diagnostic>;

fn codegen_error(message: impl Into<String>, span: Span, id: DiagnosticId) -> Diagnostic {
    Diagnostic::error(message.into(), span).with_id(id)
}

#[derive(Debug, Clone)]
struct StringLower {
    values: Vec<String>,
    offsets: Vec<u32>,
    segments: Vec<(u32, Vec<u8>)>,
    min_pages: u32,
    heap_base: u32,
}

impl StringLower {
    fn offset(&self, idx: u32) -> Option<u32> {
        self.offsets.get(idx as usize).copied()
    }

    fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
}

fn lower_strings(strings: &[String]) -> StringLower {
    let values = strings.to_vec();
    let mut offsets = Vec::new();
    let mut segments = Vec::new();
    // Reserve the first 8 bytes for allocator metadata (heap ptr + free list head).
    let mut cursor: u32 = 8;
    for s in strings {
        cursor = align_to(cursor, 4);
        offsets.push(cursor);
        let mut data = Vec::new();
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(bytes);
        segments.push((cursor, data));
        cursor = cursor.saturating_add(4 + len);
    }
    let heap_base = align_to(cursor, 4);
    let min_pages = ((heap_base + 0xFFFF) / 0x10000).max(1);
    StringLower {
        values,
        offsets,
        segments,
        min_pages,
        heap_base,
    }
}

fn align_to(x: u32, align: u32) -> u32 {
    let mask = align - 1;
    (x + mask) & !mask
}

fn mapped_type_id(ctx: &TypeCtx, ty: TypeId, mapping: &BTreeMap<TypeId, TypeId>) -> TypeId {
    let ty = ctx.resolve_id(ty);
    ctx.resolve_named_type_id(mapping.get(&ty).copied().unwrap_or(ty))
}

fn extend_type_mapping(
    ctx: &TypeCtx,
    parent: &BTreeMap<TypeId, TypeId>,
    type_params: &[TypeId],
    args: &[TypeId],
) -> BTreeMap<TypeId, TypeId> {
    let mut mapping = parent.clone();
    for (param, arg) in type_params.iter().copied().zip(args.iter().copied()) {
        mapping.insert(ctx.resolve_id(param), mapped_type_id(ctx, arg, parent));
    }
    mapping
}

fn type_storage_align_bytes(ctx: &TypeCtx, ty: TypeId) -> u32 {
    type_storage_align_bytes_mapped(ctx, ty, &BTreeMap::new())
}

fn type_storage_align_bytes_mapped(
    ctx: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> u32 {
    let ty = mapped_type_id(ctx, ty, mapping);
    match ctx.get(ty) {
        TypeKind::U8 => 1,
        TypeKind::Named(name) if name == "i64" || name == "u64" || name == "f64" => 8,
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .map(|field| type_storage_align_bytes_mapped(ctx, *field, mapping))
            .max()
            .unwrap_or(1),
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| type_storage_align_bytes_mapped(ctx, *item, mapping))
            .max()
            .unwrap_or(1),
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .filter_map(|variant| variant.payload)
            .map(|payload| type_storage_align_bytes_mapped(ctx, payload, mapping))
            .max()
            .unwrap_or(4)
            .max(4),
        TypeKind::Apply { base, args } => {
            let base = ctx.resolve_named_type_id(base);
            match ctx.get(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(ctx, mapping, &type_params, &args);
                    fields
                        .iter()
                        .map(|field| type_storage_align_bytes_mapped(ctx, *field, &nested_mapping))
                        .max()
                        .unwrap_or(1)
                }
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(ctx, mapping, &type_params, &args);
                    variants
                        .iter()
                        .filter_map(|variant| variant.payload)
                        .map(|payload| {
                            type_storage_align_bytes_mapped(ctx, payload, &nested_mapping)
                        })
                        .max()
                        .unwrap_or(4)
                        .max(4)
                }
                TypeKind::Tuple { items } => items
                    .iter()
                    .map(|item| type_storage_align_bytes_mapped(ctx, *item, mapping))
                    .max()
                    .unwrap_or(1),
                _ => 4,
            }
        }
        _ => 4,
    }
}

fn type_storage_size_bytes(ctx: &TypeCtx, ty: TypeId) -> u32 {
    type_storage_size_bytes_mapped(ctx, ty, &BTreeMap::new())
}

fn type_storage_size_bytes_mapped(
    ctx: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> u32 {
    let ty = mapped_type_id(ctx, ty, mapping);
    match ctx.get(ty) {
        TypeKind::Unit | TypeKind::Never => 0,
        TypeKind::U8 => 1,
        TypeKind::Named(name) if name == "i64" || name == "u64" || name == "f64" => 8,
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .map(|field| type_storage_size_bytes_mapped(ctx, *field, mapping))
            .sum(),
        TypeKind::Tuple { items } => items
            .iter()
            .map(|item| type_storage_size_bytes_mapped(ctx, *item, mapping))
            .sum(),
        TypeKind::Enum { variants, .. } => {
            let payload = variants
                .iter()
                .filter_map(|variant| variant.payload)
                .map(|payload| type_storage_size_bytes_mapped(ctx, payload, mapping))
                .max()
                .unwrap_or(0);
            4 + payload
        }
        TypeKind::Apply { base, args } => {
            let base = ctx.resolve_named_type_id(base);
            match ctx.get(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(ctx, mapping, &type_params, &args);
                    fields
                        .iter()
                        .map(|field| type_storage_size_bytes_mapped(ctx, *field, &nested_mapping))
                        .sum()
                }
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(ctx, mapping, &type_params, &args);
                    let payload = variants
                        .iter()
                        .filter_map(|variant| variant.payload)
                        .map(|payload| {
                            type_storage_size_bytes_mapped(ctx, payload, &nested_mapping)
                        })
                        .max()
                        .unwrap_or(0);
                    4 + payload
                }
                TypeKind::Tuple { items } => items
                    .iter()
                    .map(|item| type_storage_size_bytes_mapped(ctx, *item, mapping))
                    .sum(),
                _ => 4,
            }
        }
        _ => 4,
    }
}

fn is_aggregate_storage_type(ctx: &TypeCtx, ty: TypeId) -> bool {
    let ty = ctx.resolve_named_type_id(ty);
    match ctx.get(ty) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } | TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => matches!(
            ctx.get(ctx.resolve_named_type_id(base)),
            TypeKind::Struct { .. } | TypeKind::Tuple { .. } | TypeKind::Enum { .. }
        ),
        _ => false,
    }
}

fn tuple_field_layout(ctx: &TypeCtx, ty: TypeId, index: usize) -> Option<(TypeId, u32)> {
    let ty = ctx.resolve_named_type_id(ty);
    match ctx.get(ty) {
        TypeKind::Tuple { items } => {
            let item_ty = *items.get(index)?;
            let offset = items[..index]
                .iter()
                .map(|item| type_storage_size_bytes(ctx, *item))
                .sum();
            Some((item_ty, offset))
        }
        TypeKind::Apply { base, .. } => tuple_field_layout(ctx, base, index),
        _ => None,
    }
}

fn tuple_field_layouts_by_result(
    ctx: &TypeCtx,
    ty: TypeId,
    result_ty: TypeId,
) -> Vec<(u32, TypeId, u32)> {
    let ty = ctx.resolve_named_type_id(ty);
    match ctx.get(ty) {
        TypeKind::Tuple { items } => {
            let mut out = Vec::new();
            let mut offset = 0u32;
            let want = ctx.resolve_named_type_id(result_ty);
            for (index, item_ty) in items.iter().copied().enumerate() {
                if ctx.resolve_named_type_id(item_ty) == want {
                    out.push((index as u32, item_ty, offset));
                }
                offset += type_storage_size_bytes(ctx, item_ty);
            }
            out
        }
        TypeKind::Apply { base, .. } => tuple_field_layouts_by_result(ctx, base, result_ty),
        _ => Vec::new(),
    }
}

fn struct_field_layout_by_name(
    ctx: &TypeCtx,
    ty: TypeId,
    field_name: &str,
) -> Option<(TypeId, u32)> {
    let ty = ctx.resolve_named_type_id(ty);
    match ctx.get(ty) {
        TypeKind::Struct {
            fields,
            field_names,
            ..
        } => {
            let index = field_names.iter().position(|name| name == field_name)?;
            let field_ty = *fields.get(index)?;
            let offset = fields[..index]
                .iter()
                .map(|field| type_storage_size_bytes(ctx, *field))
                .sum();
            Some((field_ty, offset))
        }
        TypeKind::Apply { base, args } => {
            let base = ctx.resolve_named_type_id(base);
            match ctx.get(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    field_names,
                    ..
                } => {
                    let index = field_names.iter().position(|name| name == field_name)?;
                    let mapping = extend_type_mapping(ctx, &BTreeMap::new(), &type_params, &args);
                    let field_ty = mapped_type_id(ctx, *fields.get(index)?, &mapping);
                    let offset = fields[..index]
                        .iter()
                        .map(|field| type_storage_size_bytes_mapped(ctx, *field, &mapping))
                        .sum();
                    Some((field_ty, offset))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn aggregate_field_layout(
    ctx: &TypeCtx,
    base_ty: TypeId,
    field_expr: &HirExpr,
    strings: &StringLower,
) -> Option<(TypeId, u32)> {
    match &field_expr.kind {
        HirExprKind::LiteralI32(index) if *index >= 0 => {
            tuple_field_layout(ctx, base_ty, *index as usize)
        }
        HirExprKind::LiteralStr(id) => {
            let field_name = strings.values.get(*id as usize)?;
            struct_field_layout_by_name(ctx, base_ty, field_name.as_str())
        }
        _ => None,
    }
}

pub fn generate_wasm(ctx: &TypeCtx, module: &HirModule) -> Result<CodegenResult, Vec<Diagnostic>> {
    if crate::log::is_verbose() {
        let names = module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("new__"))
            .map(|f| f.name.clone())
            .collect::<Vec<_>>();
        wasm_log!("wasm codegen functions(new*): {:?}", names);
    }
    let strings = lower_strings(&module.string_literals);

    // Build imports / function list (builtins first)
    let mut imports: Vec<ImportLower> = Vec::new();
    let mut functions: Vec<FuncLower> = Vec::new();

    // Extern imports
    for ext in &module.externs {
        let Some(sig) = wasm_sig_ids(ctx, ext.result, &ext.params) else {
            return Err(vec![codegen_error(
                format!(
                    "unsupported extern signature reached wasm codegen for '{}'",
                    ext.local_name
                ),
                ext.span,
                DiagnosticId::CodegenWasmUnsupportedExternSignature,
            )]);
        };
        imports.push(ImportLower::function(
            ext.module.clone(),
            ext.name.clone(),
            ext.local_name.clone(),
            sig.0,
            sig.1,
        ));
    }

    // User functions
    for f in &module.functions {
        if crate::log::is_verbose() && f.name.contains("partition") {
            wasm_log!(
                "wasm codegen candidate partition-like: {} skip={} func_ty={}",
                f.name,
                crate::wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f),
                ctx.type_to_string(f.func_ty)
            );
        }
        if crate::wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f) {
            continue;
        }
        let Some(sig) = wasm_sig(ctx, f.result, &f.params) else {
            return Err(vec![codegen_error(
                format!(
                    "unsupported function signature reached wasm codegen for '{}'",
                    f.name
                ),
                f.span,
                DiagnosticId::CodegenWasmUnsupportedFunctionSignature,
            )]);
        };
        functions.push(FuncLower::user(f, sig));
    }

    // Map names to indices
    let mut name_to_index = BTreeMap::new();
    let mut next_index: u32 = 0;
    for imp in &imports {
        name_to_index.insert(imp.name.clone(), next_index);
        next_index += 1;
    }
    for (idx, f) in functions.iter().enumerate() {
        name_to_index.insert(f.name.clone(), next_index + idx as u32);
    }
    let total_function_slots = next_index + functions.len() as u32;

    // Type section dedup
    let mut type_section = TypeSection::new();
    let mut sig_map: BTreeMap<(Vec<ValType>, Vec<ValType>), u32> = BTreeMap::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section
                .ty()
                .function(f.params.clone(), f.results.clone());
            idx
        });
    }
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section
                .ty()
                .function(imp.params.clone(), imp.results.clone());
            idx
        });
    }
    for (params, results) in crate::wasm_shared::collect_wasm_signature_set(ctx, module) {
        let key = (params.clone(), results.clone());
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section.ty().function(params, results);
            idx
        });
    }

    let mut import_section = ImportSection::new();
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        let Some(type_idx) = sig_map.get(&key).copied() else {
            return Err(vec![codegen_error(
                format!("missing lowered wasm signature for import '{}'", imp.name),
                Span::dummy(),
                DiagnosticId::CodegenWasmMissingLoweredSignature,
            )]);
        };
        import_section.import(&imp.module, &imp.field, EntityType::Function(type_idx));
    }

    let mut func_section = FunctionSection::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        let Some(type_idx) = sig_map.get(&key).copied() else {
            return Err(vec![codegen_error(
                format!("missing lowered wasm signature for function '{}'", f.name),
                Span::dummy(),
                DiagnosticId::CodegenWasmMissingLoweredSignature,
            )]);
        };
        func_section.function(type_idx);
    }

    let mut code_section = CodeSection::new();
    for f in &functions {
        let body = lower_body(ctx, f, &name_to_index, &sig_map, &strings).map_err(|d| vec![d])?;
        code_section.function(&body);
    }

    let mut memory_section = MemorySection::new();
    memory_section.memory(MemoryType {
        minimum: strings.min_pages as u64,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });

    let mut export_section = ExportSection::new();
    export_section.export("memory", ExportKind::Memory, 0);
    if let Some(entry) = &module.entry {
        if let Some(idx) = name_to_index.get(entry) {
            export_section.export("main", ExportKind::Func, *idx);
            if entry != "main" {
                export_section.export(entry, ExportKind::Func, *idx);
            }
            export_section.export("_start", ExportKind::Func, *idx);
        }
    }

    let mut data_section = DataSection::new();
    // Store initial heap pointer (aligned end of static data) at address 0.
    data_section.active(
        0,
        &ConstExpr::i32_const(0),
        strings.heap_base.to_le_bytes().to_vec(),
    );
    data_section.active(0, &ConstExpr::i32_const(4), 0u32.to_le_bytes().to_vec());
    for (offset, bytes) in &strings.segments {
        data_section.active(0, &ConstExpr::i32_const(*offset as i32), bytes.clone());
    }

    let need_table = !sig_map.is_empty();
    let mut table_section = TableSection::new();
    let mut element_section = ElementSection::new();
    if need_table {
        table_section.table(TableType {
            element_type: RefType::FUNCREF,
            table64: false,
            minimum: total_function_slots as u64,
            maximum: Some(total_function_slots as u64),
            shared: false,
        });
        let func_indices: Vec<u32> = (0..total_function_slots).collect();
        element_section.segment(ElementSegment {
            mode: ElementMode::Active {
                table: Some(0),
                offset: &ConstExpr::i32_const(0),
            },
            elements: Elements::Functions(Cow::Owned(func_indices)),
        });
    }

    let mut module_bytes = Module::new();
    module_bytes.section(&type_section);
    if !imports.is_empty() {
        module_bytes.section(&import_section);
    }
    module_bytes.section(&func_section);
    if need_table {
        module_bytes.section(&table_section);
    }
    module_bytes.section(&memory_section);
    module_bytes.section(&export_section);
    if need_table {
        module_bytes.section(&element_section);
    }
    module_bytes.section(&code_section);
    module_bytes.section(&data_section);

    Ok(CodegenResult {
        bytes: Some(module_bytes.finish()),
        diagnostics: Vec::new(),
    })
}

pub(crate) fn should_skip_wasm_codegen_for_generic(ctx: &TypeCtx, f: &HirFunction) -> bool {
    crate::wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f)
}

fn has_unbound_type_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let resolved = ctx.resolve_id(ty);
    match ctx.get(resolved) {
        TypeKind::Var(tv) => match tv.binding {
            Some(next) => has_unbound_type_var(ctx, next),
            None => true,
        },
        TypeKind::Enum { type_params, .. } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || fields.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Tuple { items } => items.iter().any(|t| has_unbound_type_var(ctx, *t)),
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            type_params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || params.iter().any(|t| has_unbound_type_var(ctx, *t))
                || has_unbound_type_var(ctx, result)
        }
        TypeKind::Apply { base, args } => {
            has_unbound_type_var(ctx, base) || args.iter().any(|t| has_unbound_type_var(ctx, *t))
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => has_unbound_type_var(ctx, inner),
        _ => false,
    }
}

fn collect_called_functions_from_expr(
    expr: &HirExpr,
    out: &mut BTreeSet<String>,
    has_indirect: &mut bool,
) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            if let FuncRef::User(name, _) = callee {
                out.insert(name.clone());
            }
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            *has_indirect = true;
            collect_called_functions_from_expr(callee, out, has_indirect);
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_called_functions_from_expr(cond, out, has_indirect);
            collect_called_functions_from_expr(then_branch, out, has_indirect);
            collect_called_functions_from_expr(else_branch, out, has_indirect);
        }
        HirExprKind::While { cond, body } => {
            collect_called_functions_from_expr(cond, out, has_indirect);
            collect_called_functions_from_expr(body, out, has_indirect);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_called_functions_from_expr(scrutinee, out, has_indirect);
            for arm in arms {
                collect_called_functions_from_expr(&arm.body, out, has_indirect);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_called_functions_from_expr(p, out, has_indirect);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                collect_called_functions_from_expr(f, out, has_indirect);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for i in items {
                collect_called_functions_from_expr(i, out, has_indirect);
            }
        }
        HirExprKind::Block(b) => {
            for line in &b.lines {
                collect_called_functions_from_expr(&line.expr, out, has_indirect);
            }
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_called_functions_from_expr(value, out, has_indirect);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for a in args {
                collect_called_functions_from_expr(a, out, has_indirect);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_called_functions_from_expr(inner, out, has_indirect);
        }
        HirExprKind::Var(name) | HirExprKind::FnValue(name) => {
            out.insert(name.clone());
        }
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Drop { .. } => {}
    }
}

pub(crate) fn collect_reachable_wasm_functions(module: &HirModule) -> BTreeSet<String> {
    crate::wasm_shared::collect_reachable_wasm_functions(module)
}

// ---------------------------------------------------------------------
// Function lowering
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FuncLower<'a> {
    name: String,
    params: Vec<ValType>,
    results: Vec<ValType>,
    body: FuncBodyLower<'a>,
}

#[derive(Debug, Clone)]
struct ImportLower {
    module: String,
    field: String,
    name: String,
    params: Vec<ValType>,
    results: Vec<ValType>,
}

#[derive(Debug, Clone)]
enum FuncBodyLower<'a> {
    User(&'a HirFunction),
}

impl<'a> FuncLower<'a> {
    fn user(func: &'a HirFunction, sig: (Vec<ValType>, Vec<ValType>)) -> Self {
        Self {
            name: func.name.clone(),
            params: sig.0,
            results: sig.1,
            body: FuncBodyLower::User(func),
        }
    }
}

impl ImportLower {
    fn function(
        module: String,
        field: String,
        local_name: String,
        params: Vec<ValType>,
        results: Vec<ValType>,
    ) -> Self {
        Self {
            module,
            field,
            name: local_name,
            params,
            results,
        }
    }
}

pub(crate) fn wasm_sig(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[HirParam],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    crate::wasm_shared::wasm_sig(ctx, result, params)
}

pub(crate) fn wasm_sig_ids(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[TypeId],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    crate::wasm_shared::wasm_sig_ids(ctx, result, params)
}

fn valtype(kind: &TypeKind) -> Option<ValType> {
    match kind {
        TypeKind::Unit => None,
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Str => Some(ValType::I32),
        TypeKind::F32 => Some(ValType::F32),
        TypeKind::Enum { .. } | TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            Some(ValType::I32)
        }
        TypeKind::Reference(_, _) | TypeKind::Box(_) => Some(ValType::I32),
        TypeKind::Function { .. } => Some(ValType::I32),
        TypeKind::Var(_) => Some(ValType::I32),
        TypeKind::Named(name) => match name.as_str() {
            "i64" | "u64" => Some(ValType::I64),
            "f64" => Some(ValType::F64),
            _ => Some(ValType::I32),
        },
        TypeKind::Apply { .. } => {
            // std::eprintln!("valtype: Apply is Some(I32)");
            Some(ValType::I32)
        }
        other => {
            // std::eprintln!("valtype: other {:?} is None", other);
            None
        }
    }
}

fn find_alloc_index(name_map: &BTreeMap<String, u32>, current_func: &str) -> Option<u32> {
    runtime_helpers::find_runtime_helper_index(
        name_map,
        RuntimeHelperKind::Alloc,
        Some(current_func),
    )
}

pub(crate) fn collect_wasm_signature_set(
    ctx: &TypeCtx,
    module: &HirModule,
) -> BTreeSet<(Vec<ValType>, Vec<ValType>)> {
    crate::wasm_shared::collect_wasm_signature_set(ctx, module)
}

fn emit_inline_alloc(locals: &mut LocalMap, insts: &mut Vec<Instruction<'static>>) {
    let size_local = locals.alloc_temp(ValType::I32);
    let base_local = locals.alloc_temp(ValType::I32);
    let new_local = locals.alloc_temp(ValType::I32);

    // stack: [size]
    // size_local = size
    insts.push(Instruction::LocalSet(size_local));

    // base = load_i32(0)
    insts.push(Instruction::I32Const(0));
    insts.push(Instruction::I32Load(MemArg {
        offset: 0,
        align: 2,
        memory_index: 0,
    }));
    insts.push(Instruction::LocalSet(base_local));

    // new = align4(base + size)
    insts.push(Instruction::LocalGet(base_local));
    insts.push(Instruction::LocalGet(size_local));
    insts.push(Instruction::I32Add);
    insts.push(Instruction::I32Const(3));
    insts.push(Instruction::I32Add);
    insts.push(Instruction::I32Const(-4));
    insts.push(Instruction::I32And);
    insts.push(Instruction::LocalSet(new_local));

    // store_i32(0, new)
    insts.push(Instruction::I32Const(0));
    insts.push(Instruction::LocalGet(new_local));
    insts.push(Instruction::I32Store(MemArg {
        offset: 0,
        align: 2,
        memory_index: 0,
    }));

    // return base
    insts.push(Instruction::LocalGet(base_local));
}

fn emit_alloc_call(locals: &mut LocalMap, insts: &mut Vec<Instruction<'static>>) {
    if let Some(idx) = locals.alloc_helper_idx {
        insts.push(Instruction::Call(idx));
    } else {
        emit_inline_alloc(locals, insts);
    }
}

fn find_function_value_index(name_map: &BTreeMap<String, u32>, base: &str) -> Option<u32> {
    if let Some(idx) = name_map.get(base) {
        return Some(*idx);
    }
    let mut prefix = String::from(base);
    prefix.push_str("__");
    let mut found: Option<u32> = None;
    for (name, idx) in name_map {
        if name.starts_with(&prefix) {
            if found.is_some() {
                return None;
            }
            found = Some(*idx);
        }
    }
    found
}

fn lower_body<'a>(
    ctx: &TypeCtx,
    func: &FuncLower<'a>,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringLower,
) -> LowerResult<Function> {
    match func.body {
        FuncBodyLower::User(f) => lower_user(ctx, f, name_map, sig_map, strings),
    }
}

// ---------------------------------------------------------------------
// User function lowering
// ---------------------------------------------------------------------

fn lower_user(
    ctx: &TypeCtx,
    func: &HirFunction,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringLower,
) -> LowerResult<Function> {
    let mut locals = LocalMap::new();
    for p in &func.params {
        locals.register_param(p.name.clone(), p.ty, ctx);
    }
    locals.alloc_helper_idx = find_alloc_index(name_map, &func.name);

    let mut insts: Vec<Instruction<'static>> = Vec::new();

    match &func.body {
        HirBody::Block(block) => {
            let produced = gen_block(
                ctx,
                block,
                name_map,
                sig_map,
                strings,
                &mut locals,
                &mut insts,
            )?;
            let expected = valtype(&ctx.get(func.result));
            if expected.is_some() && produced.flatten().is_none() {
                return Err(codegen_error(
                    format!(
                        "function '{}' reached wasm codegen without a return value",
                        func.name
                    ),
                    func.span,
                    DiagnosticId::CodegenWasmMissingReturnValue,
                ));
            }
        }
        HirBody::Wasm(wb) => {
            for line in &wb.lines {
                match parse_wasm_line(line, &locals) {
                    Ok(mut v) => insts.append(&mut v),
                    Err(msg) => {
                        return Err(codegen_error(
                            format!(
                                "wasm raw line parse failed in function '{}': {}",
                                func.name, msg
                            ),
                            wb.span,
                            DiagnosticId::CodegenWasmRawLineParseError,
                        ));
                    }
                }
            }
        }
        HirBody::LlvmIr(_) => {
            return Err(codegen_error(
                format!(
                    "llvm ir body reached wasm codegen in function '{}'",
                    func.name
                ),
                func.span,
                DiagnosticId::CodegenWasmLlvmIrBodyNotSupported,
            ));
        }
    }

    let mut wasm_func = Function::new(locals.local_decls());
    for inst in insts {
        wasm_func.instruction(&inst);
    }
    wasm_func.instruction(&Instruction::End);
    Ok(wasm_func)
}

fn gen_block(
    ctx: &TypeCtx,
    block: &HirBlock,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<Option<ValType>>> {
    // gen_block semantics:
    // - Each `HirLine` may set `drop_result` to indicate that the
    //   value produced by that line should be dropped (emit `drop`).
    // - `drop_result` only means "drop the value produced by this line".
    // - The block's return candidate (`last_val`) is NOT destroyed by a
    //   `drop_result` line — only non-drop lines update the return candidate.
    //
    // Rationale: `drop_result` is a statement-level side effect; the
    // block return value should be managed as a separate concern so that
    // epilogue drops (or drop-inserted housekeeping) cannot accidentally
    // erase the function's return value. In future the HIR should be
    // evolved to explicitly separate `result_expr` from drop lines.
    locals.begin_scope();
    predeclare_block_locals(ctx, block, locals);
    let mut last_val: Option<ValType> = None;
    for line in &block.lines {
        let val = gen_expr(ctx, &line.expr, name_map, sig_map, strings, locals, insts)?;
        if line.drop_result {
            if val.is_some() {
                insts.push(Instruction::Drop);
            }
            // Do not clear `last_val` here. A drop on a line should
            // not erase the block's previously-known return value;
            // only non-drop lines update the `last_val` to the
            // expression's produced value.
        } else {
            last_val = val;
        }
    }
    locals.end_scope();
    Ok(Some(last_val))
}

fn predeclare_block_locals(ctx: &TypeCtx, block: &HirBlock, locals: &mut LocalMap) {
    for line in &block.lines {
        if let HirExprKind::Let { name, value, .. } = &line.expr.kind {
            let _ = locals.ensure_local(name.clone(), value.ty, ctx);
        }
    }
}

fn can_lower_simple_expr_iteratively(expr: &HirExpr) -> bool {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Var(_)
            | HirExprKind::FnValue(_)
            | HirExprKind::Drop { .. } => {}
            HirExprKind::CallIndirect { .. }
            | HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Intrinsic { .. }
            | HirExprKind::EnumConstruct { .. }
            | HirExprKind::StructConstruct { .. }
            | HirExprKind::TupleConstruct { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => return false,
        }
    }
    true
}

fn missing_direct_call_name(ctx: &TypeCtx, callee: &FuncRef) -> String {
    match callee {
        FuncRef::Builtin(n) | FuncRef::User(n, _) => n.clone(),
        FuncRef::Trait {
            trait_name,
            trait_args: _,
            method,
            self_ty,
        } => {
            let mut s = trait_name.clone();
            s.push_str("::");
            s.push_str(method);
            s.push_str(" [self=");
            s.push_str(&ctx.type_to_string(*self_ty));
            s.push(']');
            s
        }
    }
}

fn emit_direct_call(
    ctx: &TypeCtx,
    callee: &FuncRef,
    span: Span,
    name_map: &BTreeMap<String, u32>,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<()> {
    if let Some(idx) = match callee {
        FuncRef::Builtin(n) | FuncRef::User(n, _) => name_map.get(n),
        FuncRef::Trait { .. } => None,
    } {
        insts.push(Instruction::Call(*idx));
        Ok(())
    } else {
        Err(codegen_error(
            format!(
                "unknown function '{}' reached wasm codegen",
                missing_direct_call_name(ctx, callee)
            ),
            span,
            DiagnosticId::CodegenWasmUnknownFunction,
        ))
    }
}

fn emit_linear_addr_from_local(ptr_local: u32, offset: i32, insts: &mut Vec<Instruction<'static>>) {
    insts.push(Instruction::LocalGet(ptr_local));
    if offset != 0 {
        insts.push(Instruction::I32Const(offset));
        insts.push(Instruction::I32Add);
    }
}

fn emit_zero_linear_bytes(ptr_local: u32, size: i32, insts: &mut Vec<Instruction<'static>>) {
    for off in 0..size {
        emit_linear_addr_from_local(ptr_local, off, insts);
        insts.push(Instruction::I32Const(0));
        insts.push(Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
    }
}

fn emit_copy_linear_bytes(
    dst_local: u32,
    dst_offset: i32,
    src_local: u32,
    src_offset: i32,
    size: i32,
    insts: &mut Vec<Instruction<'static>>,
) {
    for off in 0..size {
        emit_linear_addr_from_local(dst_local, dst_offset + off, insts);
        emit_linear_addr_from_local(src_local, src_offset + off, insts);
        insts.push(Instruction::I32Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        insts.push(Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
    }
}

enum SimpleExprFrame<'a> {
    Expr(&'a HirExpr),
    FinishCall { callee: &'a FuncRef, span: Span },
}

fn gen_simple_expr_iteratively(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    let mut stack = Vec::new();
    stack.push(SimpleExprFrame::Expr(expr));
    while let Some(frame) = stack.pop() {
        match frame {
            SimpleExprFrame::Expr(expr) => match &expr.kind {
                HirExprKind::LiteralI32(v) => {
                    insts.push(Instruction::I32Const(*v));
                }
                HirExprKind::LiteralF32(v) => {
                    insts.push(Instruction::F32Const((*v).into()));
                }
                HirExprKind::LiteralBool(b) => {
                    insts.push(Instruction::I32Const(if *b { 1 } else { 0 }));
                }
                HirExprKind::LiteralStr(id) => {
                    if let Some(off) = strings.offset(*id) {
                        insts.push(Instruction::I32Const(off as i32));
                    } else {
                        return Err(codegen_error(
                            "string literal not found during codegen",
                            expr.span,
                            DiagnosticId::CodegenWasmStringLiteralNotFound,
                        ));
                    }
                }
                HirExprKind::Unit | HirExprKind::Drop { .. } => {}
                HirExprKind::Var(name) => {
                    if let Some(idx) = locals.lookup(name) {
                        if valtype(&ctx.get(expr.ty)).is_some() {
                            insts.push(Instruction::LocalGet(idx));
                        }
                    } else if let Some(fidx) = find_function_value_index(name_map, name) {
                        insts.push(Instruction::I32Const(fidx as i32));
                    } else {
                        return Err(codegen_error(
                            format!("unknown variable '{}' reached wasm codegen", name),
                            expr.span,
                            DiagnosticId::CodegenWasmUnknownVariable,
                        ));
                    }
                }
                HirExprKind::FnValue(name) => {
                    if let Some(fidx) = find_function_value_index(name_map, name) {
                        insts.push(Instruction::I32Const(fidx as i32));
                    } else {
                        return Err(codegen_error(
                            format!("unknown function value '{}' reached wasm codegen", name),
                            expr.span,
                            DiagnosticId::CodegenWasmUnknownFunctionValue,
                        ));
                    }
                }
                HirExprKind::Call { callee, args } => {
                    stack.push(SimpleExprFrame::FinishCall {
                        callee,
                        span: expr.span,
                    });
                    for arg in args.iter().rev() {
                        stack.push(SimpleExprFrame::Expr(arg));
                    }
                }
                HirExprKind::CallIndirect { .. }
                | HirExprKind::If { .. }
                | HirExprKind::While { .. }
                | HirExprKind::Block(_)
                | HirExprKind::Intrinsic { .. }
                | HirExprKind::EnumConstruct { .. }
                | HirExprKind::StructConstruct { .. }
                | HirExprKind::TupleConstruct { .. }
                | HirExprKind::Match { .. }
                | HirExprKind::Let { .. }
                | HirExprKind::Set { .. }
                | HirExprKind::AddrOf(_)
                | HirExprKind::Deref(_) => unreachable!("iterative wasm lowering precheck failed"),
            },
            SimpleExprFrame::FinishCall { callee, span } => {
                emit_direct_call(ctx, callee, span, name_map, insts)?;
            }
        }
    }
    Ok(valtype(&ctx.get(expr.ty)))
}

fn gen_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    if can_lower_simple_expr_iteratively(expr) {
        return gen_simple_expr_iteratively(ctx, expr, name_map, strings, locals, insts);
    }

    Ok(match &expr.kind {
        HirExprKind::LiteralI32(v) => {
            insts.push(Instruction::I32Const(*v));
            Some(ValType::I32)
        }
        HirExprKind::LiteralF32(v) => {
            insts.push(Instruction::F32Const((*v).into()));
            Some(ValType::F32)
        }
        HirExprKind::LiteralBool(b) => {
            insts.push(Instruction::I32Const(if *b { 1 } else { 0 }));
            Some(ValType::I32)
        }
        HirExprKind::LiteralStr(id) => {
            if let Some(off) = strings.offset(*id) {
                insts.push(Instruction::I32Const(off as i32));
                Some(ValType::I32)
            } else {
                return Err(codegen_error(
                    "string literal not found during codegen",
                    expr.span,
                    DiagnosticId::CodegenWasmStringLiteralNotFound,
                ));
            }
        }
        HirExprKind::Unit => None,
        HirExprKind::Var(name) => {
            if let Some(idx) = locals.lookup(name) {
                if valtype(&ctx.get(expr.ty)).is_some() {
                    insts.push(Instruction::LocalGet(idx));
                }
                valtype(&ctx.get(expr.ty))
            } else if let Some(fidx) = find_function_value_index(name_map, name) {
                // Function symbols are first-class values in HIR.
                // Lower them to table/function index constants.
                insts.push(Instruction::I32Const(fidx as i32));
                Some(ValType::I32)
            } else {
                return Err(codegen_error(
                    format!("unknown variable '{}' reached wasm codegen", name),
                    expr.span,
                    DiagnosticId::CodegenWasmUnknownVariable,
                ));
            }
        }
        HirExprKind::FnValue(name) => {
            if let Some(fidx) = find_function_value_index(name_map, name) {
                insts.push(Instruction::I32Const(fidx as i32));
                Some(ValType::I32)
            } else {
                return Err(codegen_error(
                    format!("unknown function value '{}' reached wasm codegen", name),
                    expr.span,
                    DiagnosticId::CodegenWasmUnknownFunctionValue,
                ));
            }
        }
        HirExprKind::Call { callee, args } => {
            for arg in args {
                gen_expr(ctx, arg, name_map, sig_map, strings, locals, insts)?;
            }
            emit_direct_call(ctx, callee, expr.span, name_map, insts)?;
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            result,
            args,
        } => {
            for arg in args {
                gen_expr(ctx, arg, name_map, sig_map, strings, locals, insts)?;
            }
            gen_expr(ctx, callee, name_map, sig_map, strings, locals, insts)?;
            if let Some(sig) = wasm_sig_ids(ctx, *result, params) {
                if let Some(type_idx) = sig_map.get(&sig) {
                    insts.push(Instruction::CallIndirect {
                        type_index: *type_idx,
                        table_index: 0,
                    });
                } else {
                    return Err(codegen_error(
                        "missing wasm signature for indirect call",
                        expr.span,
                        DiagnosticId::CodegenWasmMissingIndirectSignature,
                    ));
                }
            } else {
                return Err(codegen_error(
                    "unsupported indirect call signature for wasm",
                    expr.span,
                    DiagnosticId::CodegenWasmUnsupportedIndirectSignature,
                ));
            }
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            gen_expr(ctx, cond, name_map, sig_map, strings, locals, insts)?;
            let result_ty = valtype(&ctx.get(expr.ty));
            match result_ty {
                Some(vt) => insts.push(Instruction::If(wasm_encoder::BlockType::Result(vt))),
                None => insts.push(Instruction::If(wasm_encoder::BlockType::Empty)),
            }
            gen_expr(ctx, then_branch, name_map, sig_map, strings, locals, insts)?;
            insts.push(Instruction::Else);
            gen_expr(ctx, else_branch, name_map, sig_map, strings, locals, insts)?;
            insts.push(Instruction::End);
            result_ty
        }
        HirExprKind::While { cond, body } => {
            // while cond body:
            // block  ;; break target depth=1
            //   loop ;; continue target depth=0
            //     cond
            //     i32.eqz
            //     br_if 1  ;; break
            //     body
            //     br 0     ;; continue
            //   end
            // end
            insts.push(Instruction::Block(wasm_encoder::BlockType::Empty));
            insts.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
            gen_expr(ctx, cond, name_map, sig_map, strings, locals, insts)?;
            insts.push(Instruction::I32Eqz);
            insts.push(Instruction::BrIf(1));
            gen_expr(ctx, body, name_map, sig_map, strings, locals, insts)?;
            insts.push(Instruction::Br(0));
            insts.push(Instruction::End);
            insts.push(Instruction::End);
            None
        }
        HirExprKind::Block(b) => {
            gen_block(ctx, b, name_map, sig_map, strings, locals, insts)?.flatten()
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if name == "size_of" {
                let ty = type_args[0];
                let size = type_storage_size_bytes(ctx, ty) as i32;
                insts.push(Instruction::I32Const(size));
                Some(ValType::I32)
            } else if name == "align_of" {
                let ty = type_args[0];
                let align = type_storage_align_bytes(ctx, ty) as i32;
                insts.push(Instruction::I32Const(align));
                Some(ValType::I32)
            } else if name == "load" {
                let ty = type_args[0];
                let ty_kind = ctx.get(ty);
                if is_aggregate_storage_type(ctx, ty) {
                    let size = type_storage_size_bytes(ctx, ty) as i32;
                    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    insts.push(Instruction::I32Const(size));
                    emit_alloc_call(locals, insts);
                    let dst_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(dst_local));
                    for off in 0..size {
                        insts.push(Instruction::LocalGet(dst_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::LocalGet(src_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        insts.push(Instruction::I32Store8(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    }
                    insts.push(Instruction::LocalGet(dst_local));
                    return Ok(Some(ValType::I32));
                }
                let vt = valtype(&ty_kind);
                // address
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                match vt {
                    Some(ValType::I32) => {
                        if matches!(ty_kind, TypeKind::U8) {
                            insts.push(Instruction::I32Load8U(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        } else {
                            insts.push(Instruction::I32Load(MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                        }
                        Some(ValType::I32)
                    }
                    Some(ValType::F32) => {
                        insts.push(Instruction::F32Load(MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                        Some(ValType::F32)
                    }
                    Some(ValType::I64) => {
                        insts.push(Instruction::I64Load(MemArg {
                            offset: 0,
                            align: 3,
                            memory_index: 0,
                        }));
                        Some(ValType::I64)
                    }
                    Some(ValType::F64) => {
                        insts.push(Instruction::F64Load(MemArg {
                            offset: 0,
                            align: 3,
                            memory_index: 0,
                        }));
                        Some(ValType::F64)
                    }
                    None => {
                        insts.push(Instruction::Drop);
                        None
                    }
                    _ => None,
                }
            } else if name == "store" {
                let ty = type_args[0];
                let ty_kind = ctx.get(ty);
                if is_aggregate_storage_type(ctx, ty) {
                    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                    let dst_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(dst_local));
                    gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let size = type_storage_size_bytes(ctx, ty) as i32;
                    for off in 0..size {
                        insts.push(Instruction::LocalGet(dst_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::LocalGet(src_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        insts.push(Instruction::I32Store8(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    }
                    return Ok(None);
                }
                let vt = valtype(&ty_kind);

                // address
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                // value
                gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;

                match vt {
                    Some(ValType::I32) => {
                        if matches!(ty_kind, TypeKind::U8) {
                            insts.push(Instruction::I32Store8(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        } else {
                            insts.push(Instruction::I32Store(MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                        }
                        None
                    }
                    Some(ValType::F32) => {
                        insts.push(Instruction::F32Store(MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                        None
                    }
                    Some(ValType::I64) => {
                        insts.push(Instruction::I64Store(MemArg {
                            offset: 0,
                            align: 3,
                            memory_index: 0,
                        }));
                        None
                    }
                    Some(ValType::F64) => {
                        insts.push(Instruction::F64Store(MemArg {
                            offset: 0,
                            align: 3,
                            memory_index: 0,
                        }));
                        None
                    }
                    None => {
                        insts.push(Instruction::Drop);
                        insts.push(Instruction::Drop);
                        None
                    }
                    _ => None,
                }
            } else if name == "get_field" {
                if args.len() != 2 {
                    return Err(codegen_error(
                        "intrinsic get_field requires two args",
                        expr.span,
                        DiagnosticId::CodegenWasmIntrinsicArityMismatch,
                    ));
                }
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                let base_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(base_local));
                if let Some((_field_ty, offset)) =
                    aggregate_field_layout(ctx, args[0].ty, &args[1], strings)
                {
                    let field_ty = expr.ty;
                    if is_aggregate_storage_type(ctx, field_ty) {
                        let size = type_storage_size_bytes(ctx, field_ty) as i32;
                        insts.push(Instruction::I32Const(size));
                        emit_alloc_call(locals, insts);
                        let dst_local = locals.alloc_temp(ValType::I32);
                        insts.push(Instruction::LocalSet(dst_local));
                        for off in 0..size {
                            insts.push(Instruction::LocalGet(dst_local));
                            if off != 0 {
                                insts.push(Instruction::I32Const(off));
                                insts.push(Instruction::I32Add);
                            }
                            insts.push(Instruction::LocalGet(base_local));
                            insts.push(Instruction::I32Const(offset as i32 + off));
                            insts.push(Instruction::I32Add);
                            insts.push(Instruction::I32Load8U(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                            insts.push(Instruction::I32Store8(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        insts.push(Instruction::LocalGet(dst_local));
                        return Ok(Some(ValType::I32));
                    }
                    let field_kind = ctx.get(field_ty);
                    insts.push(Instruction::LocalGet(base_local));
                    if offset != 0 {
                        insts.push(Instruction::I32Const(offset as i32));
                        insts.push(Instruction::I32Add);
                    }
                    return Ok(match valtype(&field_kind) {
                        Some(ValType::I32) => {
                            if matches!(field_kind, TypeKind::U8) {
                                insts.push(Instruction::I32Load8U(MemArg {
                                    offset: 0,
                                    align: 0,
                                    memory_index: 0,
                                }));
                            } else {
                                insts.push(Instruction::I32Load(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }));
                            }
                            Some(ValType::I32)
                        }
                        Some(ValType::F32) => {
                            insts.push(Instruction::F32Load(MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                            Some(ValType::F32)
                        }
                        Some(ValType::I64) => {
                            insts.push(Instruction::I64Load(MemArg {
                                offset: 0,
                                align: 3,
                                memory_index: 0,
                            }));
                            Some(ValType::I64)
                        }
                        Some(ValType::F64) => {
                            insts.push(Instruction::F64Load(MemArg {
                                offset: 0,
                                align: 3,
                                memory_index: 0,
                            }));
                            Some(ValType::F64)
                        }
                        None => {
                            insts.push(Instruction::Drop);
                            None
                        }
                        _ => None,
                    });
                }
                let candidate_layouts = tuple_field_layouts_by_result(ctx, args[0].ty, expr.ty);
                if candidate_layouts.is_empty() {
                    return Err(codegen_error(
                        "unsupported get_field selector reached wasm codegen",
                        expr.span,
                        DiagnosticId::CodegenWasmUnsupportedFieldSelector,
                    ));
                }
                gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
                let idx_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(idx_local));
                if is_aggregate_storage_type(ctx, expr.ty) {
                    let size = type_storage_size_bytes(ctx, expr.ty) as i32;
                    insts.push(Instruction::I32Const(size));
                    emit_alloc_call(locals, insts);
                    let dst_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(dst_local));
                    for (position, _field_ty, offset) in candidate_layouts {
                        insts.push(Instruction::LocalGet(idx_local));
                        insts.push(Instruction::I32Const(position as i32));
                        insts.push(Instruction::I32Eq);
                        insts.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        for off in 0..size {
                            insts.push(Instruction::LocalGet(dst_local));
                            if off != 0 {
                                insts.push(Instruction::I32Const(off));
                                insts.push(Instruction::I32Add);
                            }
                            insts.push(Instruction::LocalGet(base_local));
                            insts.push(Instruction::I32Const(offset as i32 + off));
                            insts.push(Instruction::I32Add);
                            insts.push(Instruction::I32Load8U(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                            insts.push(Instruction::I32Store8(MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        insts.push(Instruction::End);
                    }
                    insts.push(Instruction::LocalGet(dst_local));
                    Some(ValType::I32)
                } else {
                    let out_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::I32Const(0));
                    insts.push(Instruction::LocalSet(out_local));
                    for (position, field_ty, offset) in candidate_layouts {
                        let field_kind = ctx.get(field_ty);
                        insts.push(Instruction::LocalGet(idx_local));
                        insts.push(Instruction::I32Const(position as i32));
                        insts.push(Instruction::I32Eq);
                        insts.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        insts.push(Instruction::LocalGet(base_local));
                        if offset != 0 {
                            insts.push(Instruction::I32Const(offset as i32));
                            insts.push(Instruction::I32Add);
                        }
                        match valtype(&field_kind) {
                            Some(ValType::I32) => {
                                if matches!(field_kind, TypeKind::U8) {
                                    insts.push(Instruction::I32Load8U(MemArg {
                                        offset: 0,
                                        align: 0,
                                        memory_index: 0,
                                    }));
                                } else {
                                    insts.push(Instruction::I32Load(MemArg {
                                        offset: 0,
                                        align: 2,
                                        memory_index: 0,
                                    }));
                                }
                                insts.push(Instruction::LocalSet(out_local));
                            }
                            _ => {
                                return Err(codegen_error(
                                    "unsupported runtime get_field valtype reached wasm codegen",
                                    expr.span,
                                    DiagnosticId::CodegenWasmUnsupportedFieldValueType,
                                ));
                            }
                        }
                        insts.push(Instruction::End);
                    }
                    insts.push(Instruction::LocalGet(out_local));
                    Some(ValType::I32)
                }
            } else if name == "set_field" {
                if args.len() != 3 {
                    return Err(codegen_error(
                        "intrinsic set_field requires three args",
                        expr.span,
                        DiagnosticId::CodegenWasmIntrinsicArityMismatch,
                    ));
                }
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(ctx, args[0].ty, &args[1], strings)
                else {
                    return Err(codegen_error(
                        "unsupported set_field selector reached wasm codegen",
                        expr.span,
                        DiagnosticId::CodegenWasmUnsupportedFieldSelector,
                    ));
                };
                let field_ty = args[2].ty;
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                let base_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(base_local));
                if is_aggregate_storage_type(ctx, field_ty) {
                    gen_expr(ctx, &args[2], name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let size = type_storage_size_bytes(ctx, field_ty) as i32;
                    for off in 0..size {
                        insts.push(Instruction::LocalGet(base_local));
                        insts.push(Instruction::I32Const(offset as i32 + off));
                        insts.push(Instruction::I32Add);
                        insts.push(Instruction::LocalGet(src_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        insts.push(Instruction::I32Store8(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    }
                    None
                } else {
                    let field_kind = ctx.get(field_ty);
                    insts.push(Instruction::LocalGet(base_local));
                    if offset != 0 {
                        insts.push(Instruction::I32Const(offset as i32));
                        insts.push(Instruction::I32Add);
                    }
                    gen_expr(ctx, &args[2], name_map, sig_map, strings, locals, insts)?;
                    match valtype(&field_kind) {
                        Some(ValType::I32) => {
                            if matches!(field_kind, TypeKind::U8) {
                                insts.push(Instruction::I32Store8(MemArg {
                                    offset: 0,
                                    align: 0,
                                    memory_index: 0,
                                }));
                            } else {
                                insts.push(Instruction::I32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }));
                            }
                            None
                        }
                        Some(ValType::F32) => {
                            insts.push(Instruction::F32Store(MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                            None
                        }
                        Some(ValType::I64) => {
                            insts.push(Instruction::I64Store(MemArg {
                                offset: 0,
                                align: 3,
                                memory_index: 0,
                            }));
                            None
                        }
                        Some(ValType::F64) => {
                            insts.push(Instruction::F64Store(MemArg {
                                offset: 0,
                                align: 3,
                                memory_index: 0,
                            }));
                            None
                        }
                        None => {
                            insts.push(Instruction::Drop);
                            insts.push(Instruction::Drop);
                            None
                        }
                        _ => None,
                    }
                }
            } else if name == "callsite_span" {
                let size = 12;
                insts.push(Instruction::I32Const(size));
                emit_alloc_call(locals, insts);
                let ptr_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalTee(ptr_local));

                // file_id
                insts.push(Instruction::LocalGet(ptr_local));
                insts.push(Instruction::I32Const(expr.span.file_id.0 as i32));
                insts.push(Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                // start
                insts.push(Instruction::LocalGet(ptr_local));
                insts.push(Instruction::I32Const(4));
                insts.push(Instruction::I32Add);
                insts.push(Instruction::I32Const(expr.span.start as i32));
                insts.push(Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                // end
                insts.push(Instruction::LocalGet(ptr_local));
                insts.push(Instruction::I32Const(8));
                insts.push(Instruction::I32Add);
                insts.push(Instruction::I32Const(expr.span.end as i32));
                insts.push(Instruction::I32Store(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));

                insts.push(Instruction::LocalGet(ptr_local));
                Some(ValType::I32)
            } else if name == "i32_to_f32" {
                // signed convert i32 -> f32
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::F32ConvertI32S);
                Some(ValType::F32)
            } else if name == "i32_to_u8" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::I32Const(255));
                insts.push(Instruction::I32And);
                Some(ValType::I32)
            } else if name == "i32_to_u32" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                Some(ValType::I32)
            } else if name == "f32_to_i32" {
                // signed trunc f32 -> i32
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::I32TruncF32S);
                Some(ValType::I32)
            } else if name == "u8_to_i32" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                Some(ValType::I32)
            } else if name == "u32_to_i32" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                Some(ValType::I32)
            } else if name == "i64_to_u64" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                Some(ValType::I64)
            } else if name == "u64_to_i64" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                Some(ValType::I64)
            } else if name == "reinterpret_i32_f32" {
                // bitcast i32 -> f32
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::F32ReinterpretI32);
                Some(ValType::F32)
            } else if name == "reinterpret_f32_i32" {
                // bitcast f32 -> i32
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::I32ReinterpretF32);
                Some(ValType::I32)
            } else if name == "add" {
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
                insts.push(Instruction::I32Add);
                Some(ValType::I32)
            } else if name == "unreachable" {
                insts.push(Instruction::Unreachable);
                None
            } else {
                return Err(codegen_error(
                    format!("unknown intrinsic '{}' reached wasm codegen", name),
                    expr.span,
                    DiagnosticId::CodegenWasmUnknownIntrinsic,
                ));
            }
        }
        HirExprKind::EnumConstruct {
            name: _,
            variant,
            payload,
            type_args: _,
        } => {
            let payload_offset = 4i32;
            let payload_storage_size = payload
                .as_ref()
                .map(|p| payload_offset + type_storage_size_bytes(ctx, p.ty) as i32)
                .unwrap_or(payload_offset);
            let total_size =
                (type_storage_size_bytes(ctx, expr.ty) as i32).max(payload_storage_size);
            insts.push(Instruction::I32Const(total_size));
            emit_alloc_call(locals, insts);
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalSet(ptr_local));
            emit_zero_linear_bytes(ptr_local, total_size, insts);
            emit_linear_addr_from_local(ptr_local, 0, insts);
            insts.push(Instruction::I32Const(
                enum_variant_tag(ctx, expr.ty, variant) as i32,
            ));
            insts.push(Instruction::I32Store(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            if let Some(p) = payload {
                if is_aggregate_storage_type(ctx, p.ty) {
                    gen_expr(ctx, p, name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let payload_size = type_storage_size_bytes(ctx, p.ty) as i32;
                    emit_copy_linear_bytes(
                        ptr_local,
                        payload_offset,
                        src_local,
                        0,
                        payload_size,
                        insts,
                    );
                } else {
                    match valtype(&ctx.get(p.ty)) {
                        Some(vt) => {
                            emit_linear_addr_from_local(ptr_local, payload_offset, insts);
                            gen_expr(ctx, p, name_map, sig_map, strings, locals, insts)?;
                            match vt {
                                ValType::I32 => insts.push(Instruction::I32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                })),
                                ValType::F32 => insts.push(Instruction::F32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                })),
                                ValType::I64 => insts.push(Instruction::I64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                })),
                                ValType::F64 => insts.push(Instruction::F64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                })),
                                _ => {
                                    return Err(codegen_error(
                                        "unsupported enum payload valtype reached wasm codegen",
                                        expr.span,
                                        DiagnosticId::CodegenWasmUnsupportedEnumPayloadType,
                                    ));
                                }
                            }
                        }
                        None => {
                            // Preserve side effects even when payload has no runtime representation (e.g. unit).
                            gen_expr(ctx, p, name_map, sig_map, strings, locals, insts)?;
                        }
                    }
                }
            }
            // leave pointer to constructed enum on the stack as the expression value
            insts.push(Instruction::LocalGet(ptr_local));
            Some(ValType::I32)
        }
        HirExprKind::StructConstruct {
            name: _,
            fields,
            type_args: _,
        } => {
            let mut offsets: Vec<u32> = Vec::with_capacity(fields.len());
            let mut size: u32 = 0;
            for f in fields.iter() {
                offsets.push(size);
                size += type_storage_size_bytes(ctx, f.ty);
            }
            insts.push(Instruction::I32Const(size as i32));
            emit_alloc_call(locals, insts);
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalTee(ptr_local));
            for (i, f) in fields.iter().enumerate() {
                let offset = offsets[i];
                let field_ty = f.ty;
                let vk = ctx.get(field_ty);
                if is_aggregate_storage_type(ctx, field_ty) {
                    gen_expr(ctx, f, name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let field_size = type_storage_size_bytes(ctx, field_ty) as i32;
                    for off in 0..field_size {
                        insts.push(Instruction::LocalGet(ptr_local));
                        insts.push(Instruction::I32Const(offset as i32 + off));
                        insts.push(Instruction::I32Add);
                        insts.push(Instruction::LocalGet(src_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        insts.push(Instruction::I32Store8(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    }
                    continue;
                }
                match valtype(&vk) {
                    Some(vt) => {
                        let temp = locals.alloc_temp(vt);
                        gen_expr(ctx, f, name_map, sig_map, strings, locals, insts)?;
                        insts.push(Instruction::LocalSet(temp));
                        insts.push(Instruction::LocalGet(ptr_local));
                        insts.push(Instruction::I32Const(offset as i32));
                        insts.push(Instruction::I32Add);
                        match vt {
                            ValType::I32 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::I32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }))
                            }
                            ValType::F32 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::F32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }))
                            }
                            ValType::I64 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::I64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                }))
                            }
                            ValType::F64 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::F64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                }))
                            }
                            _ => {
                                return Err(codegen_error(
                                    "unsupported struct field valtype reached wasm codegen",
                                    expr.span,
                                    DiagnosticId::CodegenWasmUnsupportedStructFieldType,
                                ));
                            }
                        }
                    }
                    None => {
                        // unit field は storage を持たないため、副作用だけ評価する。
                        gen_expr(ctx, f, name_map, sig_map, strings, locals, insts)?;
                    }
                }
            }
            Some(ValType::I32)
        }
        HirExprKind::TupleConstruct { items } => {
            let mut offsets: Vec<u32> = Vec::with_capacity(items.len());
            let mut size: u32 = 0;
            for item in items.iter() {
                offsets.push(size);
                size += type_storage_size_bytes(ctx, item.ty);
            }
            insts.push(Instruction::I32Const(size as i32));
            emit_alloc_call(locals, insts);
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalTee(ptr_local));
            for (i, item) in items.iter().enumerate() {
                let offset = offsets[i];
                let item_ty = item.ty;
                let vk = ctx.get(item_ty);
                if is_aggregate_storage_type(ctx, item_ty) {
                    gen_expr(ctx, item, name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let item_size = type_storage_size_bytes(ctx, item_ty) as i32;
                    for off in 0..item_size {
                        insts.push(Instruction::LocalGet(ptr_local));
                        insts.push(Instruction::I32Const(offset as i32 + off));
                        insts.push(Instruction::I32Add);
                        insts.push(Instruction::LocalGet(src_local));
                        if off != 0 {
                            insts.push(Instruction::I32Const(off));
                            insts.push(Instruction::I32Add);
                        }
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        insts.push(Instruction::I32Store8(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    }
                    continue;
                }
                match valtype(&vk) {
                    Some(vt) => {
                        let temp = locals.alloc_temp(vt);
                        gen_expr(ctx, item, name_map, sig_map, strings, locals, insts)?;
                        insts.push(Instruction::LocalSet(temp));
                        insts.push(Instruction::LocalGet(ptr_local));
                        insts.push(Instruction::I32Const(offset as i32));
                        insts.push(Instruction::I32Add);
                        match vt {
                            ValType::I32 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::I32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }))
                            }
                            ValType::F32 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::F32Store(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                }))
                            }
                            ValType::I64 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::I64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                }))
                            }
                            ValType::F64 => {
                                insts.push(Instruction::LocalGet(temp));
                                insts.push(Instruction::F64Store(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                }))
                            }
                            _ => {
                                return Err(codegen_error(
                                    "unsupported tuple element valtype reached wasm codegen",
                                    expr.span,
                                    DiagnosticId::CodegenWasmUnsupportedTupleElementType,
                                ));
                            }
                        }
                    }
                    None => {
                        // Unit takes 0 bytes in the tuple layout; just evaluate for side effects.
                        gen_expr(ctx, item, name_map, sig_map, strings, locals, insts)?;
                    }
                }
            }
            Some(ValType::I32)
        }
        HirExprKind::Match { scrutinee, arms } => {
            // evaluate scrutinee pointer once
            gen_expr(ctx, scrutinee, name_map, sig_map, strings, locals, insts)?;
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalSet(ptr_local));
            let result_ty = valtype(&ctx.get(expr.ty));
            insts.push(Instruction::Block(match result_ty {
                Some(vt) => wasm_encoder::BlockType::Result(vt),
                None => wasm_encoder::BlockType::Empty,
            }));
            if arms.is_empty() {
                insts.push(Instruction::Unreachable);
                insts.push(Instruction::End);
                return Ok(result_ty);
            }

            let tag_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalGet(ptr_local));
            insts.push(Instruction::I32Load(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            insts.push(Instruction::LocalSet(tag_local));

            for (idx, arm) in arms.iter().enumerate() {
                let is_last = idx + 1 == arms.len();
                let tag = enum_variant_tag(ctx, scrutinee.ty, &arm.variant);
                insts.push(Instruction::LocalGet(tag_local));
                insts.push(Instruction::I32Const(tag as i32));
                insts.push(Instruction::I32Eq);
                insts.push(Instruction::If(match result_ty {
                    Some(vt) => wasm_encoder::BlockType::Result(vt),
                    None => wasm_encoder::BlockType::Empty,
                }));
                if let Some(bind) = &arm.bind_local {
                    if let Some(payload_ty) = enum_variant_payload(ctx, scrutinee.ty, &arm.variant)
                    {
                        let lidx = locals.ensure_local(bind.clone(), payload_ty, ctx);
                        let payload_offset = 4i32;
                        if is_aggregate_storage_type(ctx, payload_ty) {
                            let payload_size = type_storage_size_bytes(ctx, payload_ty) as i32;
                            insts.push(Instruction::I32Const(payload_size));
                            emit_alloc_call(locals, insts);
                            let dst_local = locals.alloc_temp(ValType::I32);
                            insts.push(Instruction::LocalSet(dst_local));
                            emit_copy_linear_bytes(
                                dst_local,
                                0,
                                ptr_local,
                                payload_offset,
                                payload_size,
                                insts,
                            );
                            insts.push(Instruction::LocalGet(dst_local));
                            insts.push(Instruction::LocalSet(lidx));
                        } else if let Some(vt) = valtype(&ctx.get(payload_ty)) {
                            emit_linear_addr_from_local(ptr_local, payload_offset, insts);
                            match vt {
                                ValType::I32 => insts.push(Instruction::I32Load(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                })),
                                ValType::F32 => insts.push(Instruction::F32Load(MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                })),
                                ValType::I64 => insts.push(Instruction::I64Load(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                })),
                                ValType::F64 => insts.push(Instruction::F64Load(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                })),
                                _ => {
                                    return Err(codegen_error(
                                        "unsupported enum payload valtype in match reached wasm codegen",
                                        expr.span,
                                        DiagnosticId::CodegenWasmUnsupportedEnumPayloadType,
                                    ));
                                }
                            }
                            insts.push(Instruction::LocalSet(lidx));
                        }
                    }
                }
                gen_expr(ctx, &arm.body, name_map, sig_map, strings, locals, insts)?;
                if is_last {
                    insts.push(Instruction::Else);
                    insts.push(Instruction::Unreachable);
                    insts.push(Instruction::End);
                } else {
                    insts.push(Instruction::Else);
                }
            }

            for _ in 0..(arms.len() - 1) {
                insts.push(Instruction::End);
            }
            insts.push(Instruction::End);
            result_ty
        }
        HirExprKind::Let { name, value, .. } => {
            let idx = locals.ensure_local(name.clone(), value.ty, ctx);
            gen_expr(ctx, value, name_map, sig_map, strings, locals, insts)?;
            if valtype(&ctx.get(value.ty)).is_some() {
                insts.push(Instruction::LocalSet(idx));
            }
            None
        }
        HirExprKind::Set { name, value } => {
            if let Some(idx) = locals.lookup(name) {
                gen_expr(ctx, value, name_map, sig_map, strings, locals, insts)?;
                if valtype(&ctx.get(value.ty)).is_some() {
                    insts.push(Instruction::LocalSet(idx));
                }
            } else {
                return Err(codegen_error(
                    format!("unknown variable '{}' in set reached wasm codegen", name),
                    expr.span,
                    DiagnosticId::CodegenWasmUnknownVariable,
                ));
            }
            None
        }
        HirExprKind::Drop { .. } => {
            // For now, Drop is a no-op at the wasm level.
            None
        }
        HirExprKind::AddrOf(inner) => {
            gen_expr(ctx, inner, name_map, sig_map, strings, locals, insts)?;
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::Deref(inner) => {
            let ty = ctx.resolve_id(expr.ty);
            if is_aggregate_storage_type(ctx, ty) {
                let size = type_storage_size_bytes(ctx, ty) as i32;
                gen_expr(ctx, inner, name_map, sig_map, strings, locals, insts)?;
                let src_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(src_local));
                insts.push(Instruction::I32Const(size));
                emit_alloc_call(locals, insts);
                let dst_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(dst_local));
                for off in 0..size {
                    insts.push(Instruction::LocalGet(dst_local));
                    if off != 0 {
                        insts.push(Instruction::I32Const(off));
                        insts.push(Instruction::I32Add);
                    }
                    insts.push(Instruction::LocalGet(src_local));
                    if off != 0 {
                        insts.push(Instruction::I32Const(off));
                        insts.push(Instruction::I32Add);
                    }
                    insts.push(Instruction::I32Load8U(MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                    insts.push(Instruction::I32Store8(MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                }
                insts.push(Instruction::LocalGet(dst_local));
                return Ok(Some(ValType::I32));
            }
            let ty_kind = ctx.get(ty);
            let vt = valtype(&ty_kind);
            let addr_vt = gen_expr(ctx, inner, name_map, sig_map, strings, locals, insts)?;
            match vt {
                Some(ValType::I32) => {
                    if matches!(ty_kind, TypeKind::U8) {
                        insts.push(Instruction::I32Load8U(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                    } else {
                        insts.push(Instruction::I32Load(MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                    }
                    Some(ValType::I32)
                }
                Some(ValType::F32) => {
                    insts.push(Instruction::F32Load(MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                    Some(ValType::F32)
                }
                Some(ValType::I64) => {
                    insts.push(Instruction::I64Load(MemArg {
                        offset: 0,
                        align: 3,
                        memory_index: 0,
                    }));
                    Some(ValType::I64)
                }
                Some(ValType::F64) => {
                    insts.push(Instruction::F64Load(MemArg {
                        offset: 0,
                        align: 3,
                        memory_index: 0,
                    }));
                    Some(ValType::F64)
                }
                None => {
                    if addr_vt.is_some() {
                        insts.push(Instruction::Drop);
                    }
                    None
                }
                _ => None,
            }
        }
    })
}

// ---------------------------------------------------------------------
// Locals
// ---------------------------------------------------------------------

#[derive(Debug)]
struct LocalInfo {
    name: String,
    idx: u32,
    ty: Option<TypeId>,
    is_param: bool,
}

#[derive(Debug)]
struct LocalMap {
    locals: Vec<LocalInfo>,
    map: BTreeMap<String, Vec<u32>>,
    scopes: Vec<Vec<String>>,
    next_idx: u32,
    decls: Vec<ValType>,
    alloc_helper_idx: Option<u32>,
}

impl LocalMap {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            map: BTreeMap::new(),
            scopes: vec![Vec::new()],
            next_idx: 0,
            decls: Vec::new(),
            alloc_helper_idx: None,
        }
    }

    fn register_param(&mut self, name: String, ty: TypeId, ctx: &TypeCtx) {
        let idx = if valtype(&ctx.get(ctx.resolve_id(ty))).is_some() {
            let idx = self.next_idx;
            self.next_idx += 1;
            idx
        } else {
            0
        };
        self.locals.push(LocalInfo {
            name: name.clone(),
            idx,
            ty: Some(ty),
            is_param: true,
        });
        self.bind_name(name, idx);
    }

    fn ensure_local(&mut self, name: String, ty: TypeId, ctx: &TypeCtx) -> u32 {
        if let Some(idx) = self.lookup_current(&name) {
            idx
        } else {
            let vt = valtype(&ctx.get(ty));
            let idx = if let Some(vt) = vt {
                let idx = self.next_idx;
                self.next_idx += 1;
                self.decls.push(vt);
                idx
            } else {
                // Zero-sized/unit locals do not need a wasm local slot.
                0
            };
            self.locals.push(LocalInfo {
                name: name.clone(),
                idx,
                ty: Some(ty),
                is_param: false,
            });
            self.bind_name(name, idx);
            idx
        }
    }

    fn alloc_temp(&mut self, vt: ValType) -> u32 {
        let idx = self.next_idx;
        self.next_idx += 1;
        self.locals.push(LocalInfo {
            name: format!("$t{}", idx),
            idx,
            ty: None,
            is_param: false,
        });
        self.decls.push(vt);
        idx
    }

    fn lookup(&self, name: &str) -> Option<u32> {
        self.map.get(name).and_then(|stack| stack.last().copied())
    }

    fn local_decls(&self) -> Vec<(u32, ValType)> {
        self.decls.iter().map(|v| (1u32, *v)).collect()
    }

    fn valtype_of(&self, idx: u32, ctx: &TypeCtx) -> Option<ValType> {
        self.locals
            .iter()
            .find(|l| l.idx == idx)
            .and_then(|l| l.ty.and_then(|t| valtype(&ctx.get(t))))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn end_scope(&mut self) {
        if let Some(names) = self.scopes.pop() {
            for name in names {
                let remove_entry = if let Some(stack) = self.map.get_mut(&name) {
                    stack.pop();
                    stack.is_empty()
                } else {
                    false
                };
                if remove_entry {
                    self.map.remove(&name);
                }
            }
        }
    }

    fn lookup_current(&self, name: &str) -> Option<u32> {
        let current = self.scopes.last()?;
        if current.iter().any(|n| n == name) {
            self.lookup(name)
        } else {
            None
        }
    }

    fn bind_name(&mut self, name: String, idx: u32) {
        self.map.entry(name.clone()).or_default().push(idx);
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(name);
        }
    }
}

// ---------------------------------------------------------------------
// Minimal wasm text parser for #wasm blocks
// ---------------------------------------------------------------------

fn parse_wasm_line(line: &str, locals: &LocalMap) -> Result<Vec<Instruction<'static>>, String> {
    crate::wasm_shared::parse_wasm_line_with_lookup(line, |name| locals.lookup(name))
}

pub(crate) fn is_supported_wasm_intrinsic(name: &str) -> bool {
    crate::wasm_shared::is_supported_wasm_intrinsic(name)
}

fn enum_variant_tag(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> u32 {
    let name = if let Some(pos) = variant.rfind("::") {
        &variant[pos + 2..]
    } else {
        variant
    };
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .position(|v| v.name == name)
            .map(|i| i as u32)
            .unwrap_or(0),
        TypeKind::Apply { base, .. } => enum_variant_tag(ctx, base, name),
        _ => 0,
    }
}

fn enum_variant_payload(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> Option<TypeId> {
    let name = if let Some(pos) = variant.rfind("::") {
        &variant[pos + 2..]
    } else {
        variant
    };
    let enum_ty = ctx.resolve_named_type_id(enum_ty);
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.payload),
        TypeKind::Apply { base, args } => match ctx.get(ctx.resolve_named_type_id(base)) {
            TypeKind::Enum {
                variants,
                type_params,
                ..
            } => {
                let payload = variants
                    .iter()
                    .find(|v| v.name == name)
                    .and_then(|v| v.payload);
                payload.map(|pty| {
                    if let Some(pos) = type_params.iter().position(|tp| *tp == pty) {
                        if let Some(arg) = args.get(pos) {
                            return *arg;
                        }
                    }
                    pty
                })
            }
            _ => None,
        },
        _ => None,
    }
}
