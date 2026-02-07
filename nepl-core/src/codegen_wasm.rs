//! WASM backend for NEPLG2.

#![no_std]
extern crate alloc;
extern crate std;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module,
    TypeSection, ValType,
};

use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug)]
pub struct CodegenResult {
    pub bytes: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct StringLower {
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

pub fn generate_wasm(ctx: &TypeCtx, module: &HirModule) -> CodegenResult {
    let mut diags = Vec::new();
    let strings = lower_strings(&module.string_literals);

    // Build imports / function list (builtins first)
    let mut imports: Vec<ImportLower> = Vec::new();
    let mut functions: Vec<FuncLower> = Vec::new();

    // Extern imports
    for ext in &module.externs {
        if let Some(sig) = wasm_sig_ids(ctx, ext.result, &ext.params) {
            imports.push(ImportLower::function(
                ext.module.clone(),
                ext.name.clone(),
                ext.local_name.clone(),
                sig.0,
                sig.1,
            ));
        } else {
            diags.push(Diagnostic::error(
                "unsupported extern signature for wasm",
                ext.span,
            ));
        }
    }

    // User functions
    for f in &module.functions {
        if let Some(sig) = wasm_sig(ctx, f.result, &f.params) {
            functions.push(FuncLower::user(f, sig));
        } else {
            if crate::log::is_verbose() {
                std::eprintln!(
                    "codegen: failed to lower signature for {}: result={:?}, params={:?}",
                    f.name,
                    ctx.get(f.result),
                    f.params.iter().map(|p| ctx.get(p.ty)).collect::<Vec<_>>()
                );
            }
            diags.push(Diagnostic::error(
                "unsupported function signature for wasm",
                f.span,
            ));
        }
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

    // Type section dedup
    let mut type_section = TypeSection::new();
    let mut sig_map: BTreeMap<(Vec<ValType>, Vec<ValType>), u32> = BTreeMap::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section.ty().function(f.params.clone(), f.results.clone());
            idx
        });
    }
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section.ty().function(imp.params.clone(), imp.results.clone());
            idx
        });
    }

    let mut import_section = ImportSection::new();
    for imp in &imports {
        let key = (imp.params.clone(), imp.results.clone());
        let type_idx = *sig_map.get(&key).unwrap();
        import_section.import(&imp.module, &imp.field, EntityType::Function(type_idx));
    }

    let mut func_section = FunctionSection::new();
    for f in &functions {
        let key = (f.params.clone(), f.results.clone());
        let type_idx = *sig_map.get(&key).unwrap();
        func_section.function(type_idx);
    }

    let mut code_section = CodeSection::new();
    for f in &functions {
        match lower_body(ctx, f, &name_to_index, &strings) {
            Ok(body) => {
                code_section.function(&body);
            }
            Err(mut ds) => {
                diags.append(&mut ds);
            }
        }
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

    if diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return CodegenResult {
            bytes: None,
            diagnostics: diags,
        };
    }

    let mut module_bytes = Module::new();
    module_bytes.section(&type_section);
    if !imports.is_empty() {
        module_bytes.section(&import_section);
    }
    module_bytes.section(&func_section);
    module_bytes.section(&memory_section);
    module_bytes.section(&export_section);
    module_bytes.section(&code_section);
    module_bytes.section(&data_section);

    CodegenResult {
        bytes: Some(module_bytes.finish()),
        diagnostics: diags,
    }
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

fn wasm_sig(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[HirParam],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(p.ty);
        if let Some(v) = valtype(&vk) {
            param_types.push(v);
        } else {
            if crate::log::is_verbose() {
                std::eprintln!("wasm_sig: rejected param {} with type {:?}", p.name, vk);
            }
            return None;
        }
    }
    let res_kind = ctx.get(result);
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            if crate::log::is_verbose() {
                std::eprintln!("wasm_sig: rejected result type {:?}", res_kind);
            }
            return None;
        }
        // unit return is fine in wasm
        Vec::new()
    };
    Some((param_types, res))
}

fn wasm_sig_ids(
    ctx: &TypeCtx,
    result: TypeId,
    params: &[TypeId],
) -> Option<(Vec<ValType>, Vec<ValType>)> {
    let mut param_types = Vec::new();
    for p in params {
        let vk = ctx.get(*p);
        if let Some(v) = valtype(&vk) {
            param_types.push(v);
        } else {
            return None;
        }
    }
    let res_kind = ctx.get(result);
    if crate::log::is_verbose() {
        std::eprintln!(
            "wasm_sig: checking result type {:?} with valtype={:?}",
            res_kind,
            valtype(&res_kind)
        );
    }
    let res = if let Some(v) = valtype(&res_kind) {
        vec![v]
    } else {
        if !matches!(res_kind, TypeKind::Unit) {
            if crate::log::is_verbose() {
                std::eprintln!("wasm_sig: REJECTED result type {:?}", res_kind);
            }
            return None;
        }
        // unit return is fine in wasm
        Vec::new()
    };
    Some((param_types, res))
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
        TypeKind::Named(name) => match name.as_str() {
            "i64" => Some(ValType::I64),
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

fn find_alloc_index(name_map: &BTreeMap<String, u32>) -> Option<u32> {
    if let Some(idx) = name_map.get("alloc") {
        return Some(*idx);
    }
    name_map
        .iter()
        .find(|(name, _)| name.starts_with("alloc__"))
        .map(|(_, idx)| *idx)
}

fn lower_body<'a>(
    ctx: &TypeCtx,
    func: &FuncLower<'a>,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
) -> Result<Function, Vec<Diagnostic>> {
    match func.body {
        FuncBodyLower::User(f) => lower_user(ctx, f, name_map, strings),
    }
}

// ---------------------------------------------------------------------
// User function lowering
// ---------------------------------------------------------------------

fn lower_user(
    ctx: &TypeCtx,
    func: &HirFunction,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
) -> Result<Function, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let mut locals = LocalMap::new(func.params.len());
    for p in &func.params {
        locals.register_param(p.name.clone(), p.ty);
    }

    let mut insts: Vec<Instruction<'static>> = Vec::new();

    match &func.body {
        HirBody::Block(block) => {
            let produced = gen_block(
                ctx,
                block,
                name_map,
                strings,
                &mut locals,
                &mut insts,
                &mut diags,
            );
            let expected = valtype(&ctx.get(func.result));
            if expected.is_some() && produced.flatten().is_none() {
                diags.push(Diagnostic::error(
                    "function expected to return value",
                    func.span,
                ));
                // Dump the HIR for this function to aid debugging of
                // missing-return problems. This is only emitted when the
                // error occurs so it doesn't clutter normal output.
                let mut dump = String::new();
                for (i, line) in block.lines.iter().enumerate() {
                    let kind = format!("{:?}", &line.expr.kind);
                    let ty = format!("{:?}", ctx.get(line.expr.ty));
                    let entry = format!(
                        "line {}: kind={} ty={} drop_result={}\n",
                        i, kind, ty, line.drop_result
                    );
                    dump.push_str(&entry);
                }
                diags.push(Diagnostic::warning(
                    format!("HIR dump for {}:\n{}", func.name, dump),
                    func.span,
                ));
            }
        }
        HirBody::Wasm(wb) => {
            for line in &wb.lines {
                match parse_wasm_line(line, &locals) {
                    Ok(mut v) => insts.append(&mut v),
                    Err(msg) => diags.push(Diagnostic::error(msg, func.span)),
                }
            }
            if diags.is_empty() {
                if let Err(d) = validate_wasm_stack(ctx, func, &locals, &insts) {
                    diags.push(d);
                }
            }
        }
    }

    let mut wasm_func = Function::new(locals.local_decls());
    for inst in insts {
        wasm_func.instruction(&inst);
    }
    wasm_func.instruction(&Instruction::End);
    if diags.is_empty() {
        Ok(wasm_func)
    } else {
        Err(diags)
    }
}

fn gen_block(
    ctx: &TypeCtx,
    block: &HirBlock,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
    diags: &mut Vec<Diagnostic>,
) -> Option<Option<ValType>> {
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
    let mut last_val: Option<ValType> = None;
    for line in &block.lines {
        let val = gen_expr(ctx, &line.expr, name_map, strings, locals, insts, diags);
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
    Some(last_val)
}

fn gen_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    strings: &StringLower,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
    diags: &mut Vec<Diagnostic>,
) -> Option<ValType> {
    match &expr.kind {
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
                diags.push(Diagnostic::error(
                    "string literal not found during codegen",
                    expr.span,
                ));
                None
            }
        }
        HirExprKind::Unit => None,
        HirExprKind::Var(name) => {
            if let Some(idx) = locals.lookup(name) {
                if valtype(&ctx.get(expr.ty)).is_some() {
                    insts.push(Instruction::LocalGet(idx));
                }
                valtype(&ctx.get(expr.ty))
            } else if let Some(fidx) = name_map.get(name) {
                insts.push(Instruction::Call(*fidx));
                valtype(&ctx.get(expr.ty))
            } else {
                diags.push(Diagnostic::error(
                    format!("unknown variable {}", name),
                    expr.span,
                ));
                None
            }
        }
        HirExprKind::Call { callee, args } => {
            for arg in args {
                gen_expr(ctx, arg, name_map, strings, locals, insts, diags);
            }
            if let Some(idx) = match callee {
                FuncRef::Builtin(n) | FuncRef::User(n, _) => name_map.get(n),
                FuncRef::Trait { .. } => None,
            } {
                insts.push(Instruction::Call(*idx));
            } else {
                let missing = match callee {
                    FuncRef::Builtin(n) | FuncRef::User(n, _) => n.clone(),
                    FuncRef::Trait { trait_name, method, .. } => {
                        let mut s = trait_name.clone();
                        s.push_str("::");
                        s.push_str(method);
                        s
                    }
                };
                diags.push(Diagnostic::error(
                    format!("unknown function {missing}"),
                    expr.span,
                ));
            }
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            gen_expr(ctx, cond, name_map, strings, locals, insts, diags);
            let result_ty = valtype(&ctx.get(expr.ty));
            match result_ty {
                Some(vt) => insts.push(Instruction::If(wasm_encoder::BlockType::Result(vt))),
                None => insts.push(Instruction::If(wasm_encoder::BlockType::Empty)),
            }
            gen_expr(ctx, then_branch, name_map, strings, locals, insts, diags);
            insts.push(Instruction::Else);
            gen_expr(ctx, else_branch, name_map, strings, locals, insts, diags);
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
            gen_expr(ctx, cond, name_map, strings, locals, insts, diags);
            insts.push(Instruction::I32Eqz);
            insts.push(Instruction::BrIf(1));
            gen_expr(ctx, body, name_map, strings, locals, insts, diags);
            insts.push(Instruction::Br(0));
            insts.push(Instruction::End);
            insts.push(Instruction::End);
            None
        }
        HirExprKind::Block(b) => {
            gen_block(ctx, b, name_map, strings, locals, insts, diags).flatten()
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if name == "size_of" {
                let ty = type_args[0];
                let size = match ctx.get(ty) {
                    TypeKind::U8 => 1,
                    TypeKind::Named(name) if name == "i64" || name == "f64" => 8,
                    _ => match valtype(&ctx.get(ty)) {
                        Some(_) => 4,
                        None => 0,
                    },
                };
                insts.push(Instruction::I32Const(size));
                Some(ValType::I32)
            } else if name == "align_of" {
                let ty = type_args[0];
                let align = match ctx.get(ty) {
                    TypeKind::U8 => 1,
                    TypeKind::Named(name) if name == "i64" || name == "f64" => 8,
                    _ => match valtype(&ctx.get(ty)) {
                        Some(_) => 4,
                        None => 0,
                    },
                };
                insts.push(Instruction::I32Const(align));
                Some(ValType::I32)
            } else if name == "load" {
                let ty = type_args[0];
                let ty_kind = ctx.get(ty);
                let vt = valtype(&ty_kind);
                // address
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
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
                    None => {
                        insts.push(Instruction::Drop);
                        None
                    }
                    _ => None, // Only I32/F32 supported currently
                }
            } else if name == "store" {
                let ty = type_args[0];
                let ty_kind = ctx.get(ty);
                let vt = valtype(&ty_kind);
                
                // address
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                // value
                gen_expr(ctx, &args[1], name_map, strings, locals, insts, diags);

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
                    None => {
                        insts.push(Instruction::Drop);
                        insts.push(Instruction::Drop);
                        None
                    }
                    _ => None,
                }
            } else if name == "callsite_span" {
                let size = 12;
                insts.push(Instruction::I32Const(size));
                if let Some(idx) = find_alloc_index(name_map) {
                    insts.push(Instruction::Call(idx));
                } else {
                    diags.push(Diagnostic::error(
                        "alloc function not found (import std/mem)",
                        expr.span,
                    ));
                    return None;
                }
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
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                insts.push(Instruction::F32ConvertI32S);
                Some(ValType::F32)
            } else if name == "i32_to_u8" {
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                insts.push(Instruction::I32Const(255));
                insts.push(Instruction::I32And);
                Some(ValType::I32)
            } else if name == "f32_to_i32" {
                // signed trunc f32 -> i32
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                insts.push(Instruction::I32TruncF32S);
                Some(ValType::I32)
            } else if name == "u8_to_i32" {
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                Some(ValType::I32)
            } else if name == "reinterpret_i32_f32" {
                // bitcast i32 -> f32
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                insts.push(Instruction::F32ReinterpretI32);
                Some(ValType::F32)
            } else if name == "reinterpret_f32_i32" {
                // bitcast f32 -> i32
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                insts.push(Instruction::I32ReinterpretF32);
                Some(ValType::I32)
            } else if name == "add" {
                gen_expr(ctx, &args[0], name_map, strings, locals, insts, diags);
                gen_expr(ctx, &args[1], name_map, strings, locals, insts, diags);
                insts.push(Instruction::I32Add);
                Some(ValType::I32)
            } else if name == "unreachable" {
                insts.push(Instruction::Unreachable);
                None
            } else {
                diags.push(Diagnostic::error("unknown codegen intrinsic", expr.span));
                None
            }
        }
        HirExprKind::EnumConstruct {
            name: _,
            variant,
            payload,
            type_args: _,
        } => {
            let payload_vt = payload
                .as_ref()
                .and_then(|p| valtype(&ctx.get(p.ty)))
                .unwrap_or(ValType::I32);
            let size = if payload.is_some() { 8 } else { 4 };
            insts.push(Instruction::I32Const(size));
            if let Some(idx) = find_alloc_index(name_map) {
                insts.push(Instruction::Call(idx));
            } else {
                diags.push(Diagnostic::error(
                    "alloc function not found (import std/mem)",
                    expr.span,
                ));
                return None;
            }
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalTee(ptr_local));
            // store tag
            insts.push(Instruction::I32Const(
                enum_variant_tag(ctx, expr.ty, variant) as i32,
            ));
            insts.push(Instruction::I32Store(MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            if let Some(p) = payload {
                // evaluate payload and store at offset 4
                insts.push(Instruction::LocalGet(ptr_local));
                insts.push(Instruction::I32Const(4));
                insts.push(Instruction::I32Add);
                gen_expr(ctx, p, name_map, strings, locals, insts, diags);
                match payload_vt {
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
                    _ => {
                        diags.push(Diagnostic::error(
                            "unsupported enum payload type",
                            expr.span,
                        ));
                        return None;
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
            let size = (fields.len() as i32) * 4;
            insts.push(Instruction::I32Const(size));
            if let Some(idx) = find_alloc_index(name_map) {
                insts.push(Instruction::Call(idx));
            } else {
                diags.push(Diagnostic::error(
                    "alloc function not found (import std/mem)",
                    expr.span,
                ));
                return None;
            }
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalTee(ptr_local));
            for (i, f) in fields.iter().enumerate() {
                let offset = (i as u32) * 4;
                let vk = ctx.get(f.ty);
                let vt = valtype(&vk).unwrap_or(ValType::I32);
                let temp = locals.alloc_temp(vt);
                gen_expr(ctx, f, name_map, strings, locals, insts, diags);
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
                    _ => {
                        diags.push(Diagnostic::error(
                            "unsupported struct field type for codegen",
                            expr.span,
                        ));
                        return None;
                    }
                }
            }
            Some(ValType::I32)
        }
        HirExprKind::TupleConstruct { items } => {
            let size = (items.len() as i32) * 4;
            insts.push(Instruction::I32Const(size));
            if let Some(idx) = find_alloc_index(name_map) {
                insts.push(Instruction::Call(idx));
            } else {
                diags.push(Diagnostic::error(
                    "alloc function not found (import std/mem)",
                    expr.span,
                ));
                return None;
            }
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalTee(ptr_local));
            for (i, item) in items.iter().enumerate() {
                let offset = (i as u32) * 4;
                let vk = ctx.get(item.ty);
                let vt = valtype(&vk).unwrap_or(ValType::I32);
                let temp = locals.alloc_temp(vt);
                gen_expr(ctx, item, name_map, strings, locals, insts, diags);
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
                    _ => {
                        diags.push(Diagnostic::error(
                            "unsupported tuple element type for codegen",
                            expr.span,
                        ));
                        return None;
                    }
                }
            }
            Some(ValType::I32)
        }
        HirExprKind::Match { scrutinee, arms } => {
            // evaluate scrutinee pointer once
            gen_expr(ctx, scrutinee, name_map, strings, locals, insts, diags);
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
                return result_ty;
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
                    if let Some(payload_ty) = enum_variant_payload(ctx, scrutinee.ty, &arm.variant) {
                        let lidx = locals.ensure_local(bind.clone(), payload_ty, ctx);
                        insts.push(Instruction::LocalGet(ptr_local));
                        insts.push(Instruction::I32Const(4));
                        insts.push(Instruction::I32Add);
                        match valtype(&ctx.get(payload_ty)).unwrap_or(ValType::I32) {
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
                            _ => diags.push(Diagnostic::error(
                                "unsupported enum payload type",
                                arm.body.span,
                            )),
                        }
                        insts.push(Instruction::LocalSet(lidx));
                    }
                }
                gen_expr(ctx, &arm.body, name_map, strings, locals, insts, diags);
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
            gen_expr(ctx, value, name_map, strings, locals, insts, diags);
            if valtype(&ctx.get(value.ty)).is_some() {
                insts.push(Instruction::LocalSet(idx));
            }
            None
        }
        HirExprKind::Set { name, value } => {
            if let Some(idx) = locals.lookup(name) {
                gen_expr(ctx, value, name_map, strings, locals, insts, diags);
                insts.push(Instruction::LocalSet(idx));
            } else {
                diags.push(Diagnostic::error("unknown variable", expr.span));
            }
            None
        }
        HirExprKind::Drop { .. } => {
            // For now, Drop is a no-op at the wasm level.
            None
        }
        HirExprKind::AddrOf(inner) => {
            gen_expr(ctx, inner, name_map, strings, locals, insts, diags);
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::Deref(inner) => {
            gen_expr(ctx, inner, name_map, strings, locals, insts, diags);
            valtype(&ctx.get(expr.ty))
        }
    }
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
    map: BTreeMap<String, u32>,
    next_idx: u32,
    decls: Vec<ValType>,
}

impl LocalMap {
    fn new(param_count: usize) -> Self {
        Self {
            locals: Vec::new(),
            map: BTreeMap::new(),
            next_idx: param_count as u32,
            decls: Vec::new(),
        }
    }

    fn register_param(&mut self, name: String, ty: TypeId) {
        let idx = self.locals.len() as u32;
        self.locals.push(LocalInfo {
            name: name.clone(),
            idx,
            ty: Some(ty),
            is_param: true,
        });
        self.map.insert(name, idx);
    }

    fn ensure_local(&mut self, name: String, ty: TypeId, ctx: &TypeCtx) -> u32 {
        if let Some(idx) = self.lookup(&name) {
            idx
        } else {
            let idx = self.next_idx;
            self.next_idx += 1;
            self.locals.push(LocalInfo {
                name: name.clone(),
                idx,
                ty: Some(ty),
                is_param: false,
            });
            self.map.insert(name, idx);
            if let Some(vt) = valtype(&ctx.get(ty)) {
                self.decls.push(vt);
            }
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
        self.map.get(name).copied()
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
}

// ---------------------------------------------------------------------
// Minimal wasm text parser for #wasm blocks
// ---------------------------------------------------------------------

fn parse_wasm_line(line: &str, locals: &LocalMap) -> Result<Vec<Instruction<'static>>, String> {
    let mut insts = Vec::new();
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(insts);
    }
    if parts[0].starts_with(";;") {
        return Ok(insts);
    }
    match parts[0] {
        "local.get" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
                insts.push(Instruction::LocalGet(idx));
            } else {
                return Err(format!("unknown local in #wasm: {}", parts[1]));
            }
        }
        "local.set" if parts.len() == 2 => {
            if let Some(idx) = parse_local(parts[1], locals) {
                insts.push(Instruction::LocalSet(idx));
            } else {
                return Err(format!("unknown local in #wasm: {}", parts[1]));
            }
        }
        // i32 operations
        "i32.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<i32>() {
                insts.push(Instruction::I32Const(v));
            } else {
                return Err(format!("invalid i32.const immediate: {}", parts[1]));
            }
        }
        "i32.add" => insts.push(Instruction::I32Add),
        "i32.sub" => insts.push(Instruction::I32Sub),
        "i32.mul" => insts.push(Instruction::I32Mul),
        "i32.div_s" => insts.push(Instruction::I32DivS),
        "i32.div_u" => insts.push(Instruction::I32DivU),
        "i32.rem_s" => insts.push(Instruction::I32RemS),
        "i32.rem_u" => insts.push(Instruction::I32RemU),
        "i32.and" => insts.push(Instruction::I32And),
        "i32.or" => insts.push(Instruction::I32Or),
        "i32.xor" => insts.push(Instruction::I32Xor),
        "i32.shl" => insts.push(Instruction::I32Shl),
        "i32.shr_s" => insts.push(Instruction::I32ShrS),
        "i32.shr_u" => insts.push(Instruction::I32ShrU),
        "i32.rotl" => insts.push(Instruction::I32Rotl),
        "i32.rotr" => insts.push(Instruction::I32Rotr),
        "i32.clz" => insts.push(Instruction::I32Clz),
        "i32.ctz" => insts.push(Instruction::I32Ctz),
        "i32.popcnt" => insts.push(Instruction::I32Popcnt),
        "i32.eqz" => insts.push(Instruction::I32Eqz),
        "i32.eq" => insts.push(Instruction::I32Eq),
        "i32.ne" => insts.push(Instruction::I32Ne),
        "i32.lt_s" => insts.push(Instruction::I32LtS),
        "i32.lt_u" => insts.push(Instruction::I32LtU),
        "i32.le_s" => insts.push(Instruction::I32LeS),
        "i32.le_u" => insts.push(Instruction::I32LeU),
        "i32.gt_s" => insts.push(Instruction::I32GtS),
        "i32.gt_u" => insts.push(Instruction::I32GtU),
        "i32.ge_s" => insts.push(Instruction::I32GeS),
        "i32.ge_u" => insts.push(Instruction::I32GeU),
        "i32.load" => insts.push(Instruction::I32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i32.store" => insts.push(Instruction::I32Store(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i32.load8_s" => insts.push(Instruction::I32Load8S(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.load8_u" => insts.push(Instruction::I32Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.load16_s" => insts.push(Instruction::I32Load16S(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.load16_u" => insts.push(Instruction::I32Load16U(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.store8" => insts.push(Instruction::I32Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i32.store16" => insts.push(Instruction::I32Store16(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i32.extend8_s" => insts.push(Instruction::I32Extend8S),
        "i32.extend16_s" => insts.push(Instruction::I32Extend16S),
        // i64 operations
        "i64.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<i64>() {
                insts.push(Instruction::I64Const(v));
            } else {
                return Err(format!("invalid i64.const immediate: {}", parts[1]));
            }
        }
        "i64.add" => insts.push(Instruction::I64Add),
        "i64.sub" => insts.push(Instruction::I64Sub),
        "i64.mul" => insts.push(Instruction::I64Mul),
        "i64.div_s" => insts.push(Instruction::I64DivS),
        "i64.div_u" => insts.push(Instruction::I64DivU),
        "i64.rem_s" => insts.push(Instruction::I64RemS),
        "i64.rem_u" => insts.push(Instruction::I64RemU),
        "i64.and" => insts.push(Instruction::I64And),
        "i64.or" => insts.push(Instruction::I64Or),
        "i64.xor" => insts.push(Instruction::I64Xor),
        "i64.shl" => insts.push(Instruction::I64Shl),
        "i64.shr_s" => insts.push(Instruction::I64ShrS),
        "i64.shr_u" => insts.push(Instruction::I64ShrU),
        "i64.rotl" => insts.push(Instruction::I64Rotl),
        "i64.rotr" => insts.push(Instruction::I64Rotr),
        "i64.clz" => insts.push(Instruction::I64Clz),
        "i64.ctz" => insts.push(Instruction::I64Ctz),
        "i64.popcnt" => insts.push(Instruction::I64Popcnt),
        "i64.eqz" => insts.push(Instruction::I64Eqz),
        "i64.eq" => insts.push(Instruction::I64Eq),
        "i64.ne" => insts.push(Instruction::I64Ne),
        "i64.lt_s" => insts.push(Instruction::I64LtS),
        "i64.lt_u" => insts.push(Instruction::I64LtU),
        "i64.le_s" => insts.push(Instruction::I64LeS),
        "i64.le_u" => insts.push(Instruction::I64LeU),
        "i64.gt_s" => insts.push(Instruction::I64GtS),
        "i64.gt_u" => insts.push(Instruction::I64GtU),
        "i64.ge_s" => insts.push(Instruction::I64GeS),
        "i64.ge_u" => insts.push(Instruction::I64GeU),
        "i64.load" => insts.push(Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "i64.store" => insts.push(Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "i64.load8_s" => insts.push(Instruction::I64Load8S(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.load8_u" => insts.push(Instruction::I64Load8U(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.load16_s" => insts.push(Instruction::I64Load16S(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.load16_u" => insts.push(Instruction::I64Load16U(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.load32_s" => insts.push(Instruction::I64Load32S(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i64.load32_u" => insts.push(Instruction::I64Load32U(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "i64.store8" => insts.push(Instruction::I64Store8(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        })),
        "i64.store16" => insts.push(Instruction::I64Store16(MemArg {
            offset: 0,
            align: 1,
            memory_index: 0,
        })),
        "i64.store32" => insts.push(Instruction::I64Store32(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        // f32 operations
        "f32.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<f32>() {
                insts.push(Instruction::F32Const(v.into()));
            } else {
                return Err(format!("invalid f32.const immediate: {}", parts[1]));
            }
        }
        "f32.add" => insts.push(Instruction::F32Add),
        "f32.sub" => insts.push(Instruction::F32Sub),
        "f32.mul" => insts.push(Instruction::F32Mul),
        "f32.div" => insts.push(Instruction::F32Div),
        "f32.abs" => insts.push(Instruction::F32Abs),
        "f32.neg" => insts.push(Instruction::F32Neg),
        "f32.ceil" => insts.push(Instruction::F32Ceil),
        "f32.floor" => insts.push(Instruction::F32Floor),
        "f32.trunc" => insts.push(Instruction::F32Trunc),
        "f32.nearest" => insts.push(Instruction::F32Nearest),
        "f32.sqrt" => insts.push(Instruction::F32Sqrt),
        "f32.min" => insts.push(Instruction::F32Min),
        "f32.max" => insts.push(Instruction::F32Max),
        "f32.copysign" => insts.push(Instruction::F32Copysign),
        "f32.eq" => insts.push(Instruction::F32Eq),
        "f32.ne" => insts.push(Instruction::F32Ne),
        "f32.lt" => insts.push(Instruction::F32Lt),
        "f32.le" => insts.push(Instruction::F32Le),
        "f32.gt" => insts.push(Instruction::F32Gt),
        "f32.ge" => insts.push(Instruction::F32Ge),
        "f32.load" => insts.push(Instruction::F32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        "f32.store" => insts.push(Instruction::F32Store(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        })),
        // f64 operations
        "f64.const" if parts.len() == 2 => {
            if let Ok(v) = parts[1].parse::<f64>() {
                insts.push(Instruction::F64Const(v.into()));
            } else {
                return Err(format!("invalid f64.const immediate: {}", parts[1]));
            }
        }
        "f64.add" => insts.push(Instruction::F64Add),
        "f64.sub" => insts.push(Instruction::F64Sub),
        "f64.mul" => insts.push(Instruction::F64Mul),
        "f64.div" => insts.push(Instruction::F64Div),
        "f64.abs" => insts.push(Instruction::F64Abs),
        "f64.neg" => insts.push(Instruction::F64Neg),
        "f64.ceil" => insts.push(Instruction::F64Ceil),
        "f64.floor" => insts.push(Instruction::F64Floor),
        "f64.trunc" => insts.push(Instruction::F64Trunc),
        "f64.nearest" => insts.push(Instruction::F64Nearest),
        "f64.sqrt" => insts.push(Instruction::F64Sqrt),
        "f64.min" => insts.push(Instruction::F64Min),
        "f64.max" => insts.push(Instruction::F64Max),
        "f64.copysign" => insts.push(Instruction::F64Copysign),
        "f64.eq" => insts.push(Instruction::F64Eq),
        "f64.ne" => insts.push(Instruction::F64Ne),
        "f64.lt" => insts.push(Instruction::F64Lt),
        "f64.le" => insts.push(Instruction::F64Le),
        "f64.gt" => insts.push(Instruction::F64Gt),
        "f64.ge" => insts.push(Instruction::F64Ge),
        "f64.load" => insts.push(Instruction::F64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        "f64.store" => insts.push(Instruction::F64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        })),
        // Type conversions
        "i32.wrap_i64" => insts.push(Instruction::I32WrapI64),
        "i64.extend_i32_s" => insts.push(Instruction::I64ExtendI32S),
        "i64.extend_i32_u" => insts.push(Instruction::I64ExtendI32U),
        "i32.trunc_f32_s" => insts.push(Instruction::I32TruncF32S),
        "i32.trunc_f32_u" => insts.push(Instruction::I32TruncF32U),
        "i32.trunc_f64_s" => insts.push(Instruction::I32TruncF64S),
        "i32.trunc_f64_u" => insts.push(Instruction::I32TruncF64U),
        "i64.trunc_f32_s" => insts.push(Instruction::I64TruncF32S),
        "i64.trunc_f32_u" => insts.push(Instruction::I64TruncF32U),
        "i64.trunc_f64_s" => insts.push(Instruction::I64TruncF64S),
        "i64.trunc_f64_u" => insts.push(Instruction::I64TruncF64U),
        "f32.convert_i32_s" => insts.push(Instruction::F32ConvertI32S),
        "f32.convert_i32_u" => insts.push(Instruction::F32ConvertI32U),
        "f32.convert_i64_s" => insts.push(Instruction::F32ConvertI64S),
        "f32.convert_i64_u" => insts.push(Instruction::F32ConvertI64U),
        "f32.demote_f64" => insts.push(Instruction::F32DemoteF64),
        "f64.convert_i32_s" => insts.push(Instruction::F64ConvertI32S),
        "f64.convert_i32_u" => insts.push(Instruction::F64ConvertI32U),
        "f64.convert_i64_s" => insts.push(Instruction::F64ConvertI64S),
        "f64.convert_i64_u" => insts.push(Instruction::F64ConvertI64U),
        "f64.promote_f32" => insts.push(Instruction::F64PromoteF32),
        "i32.reinterpret_f32" => insts.push(Instruction::I32ReinterpretF32),
        "i64.reinterpret_f64" => insts.push(Instruction::I64ReinterpretF64),
        "f32.reinterpret_i32" => insts.push(Instruction::F32ReinterpretI32),
        "f64.reinterpret_i64" => insts.push(Instruction::F64ReinterpretI64),
        "i32.trunc_sat_f32_s" => insts.push(Instruction::I32TruncSatF32S),
        "i32.trunc_sat_f32_u" => insts.push(Instruction::I32TruncSatF32U),
        "i32.trunc_sat_f64_s" => insts.push(Instruction::I32TruncSatF64S),
        "i32.trunc_sat_f64_u" => insts.push(Instruction::I32TruncSatF64U),
        "i64.trunc_sat_f32_s" => insts.push(Instruction::I64TruncSatF32S),
        "i64.trunc_sat_f32_u" => insts.push(Instruction::I64TruncSatF32U),
        "i64.trunc_sat_f64_s" => insts.push(Instruction::I64TruncSatF64S),
        "i64.trunc_sat_f64_u" => insts.push(Instruction::I64TruncSatF64U),
        // Memory operations
        "memory.grow" => insts.push(Instruction::MemoryGrow(0)),
        "memory.size" => insts.push(Instruction::MemorySize(0)),
        "drop" => insts.push(Instruction::Drop),
        other => return Err(format!("unsupported wasm instruction: {}", other)),
    }
    Ok(insts)
}


fn parse_local(text: &str, locals: &LocalMap) -> Option<u32> {
    if let Some(stripped) = text.strip_prefix('$') {
        if let Ok(idx) = stripped.parse::<u32>() {
            Some(idx)
        } else {
            locals.lookup(stripped)
        }
    } else {
        text.parse::<u32>().ok()
    }
}

fn enum_variant_tag(ctx: &TypeCtx, enum_ty: TypeId, variant: &str) -> u32 {
    let name = if let Some(pos) = variant.rfind("::") {
        &variant[pos + 2..]
    } else {
        variant
    };
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
    match ctx.get(enum_ty) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.payload),
        TypeKind::Apply { base, args } => match ctx.get(base) {
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

fn validate_wasm_stack(
    ctx: &TypeCtx,
    func: &HirFunction,
    locals: &LocalMap,
    insts: &[Instruction<'static>],
) -> Result<(), Diagnostic> {
    // For now, we skip strict stack validation for #wasm blocks
    // The WASM runtime will validate the instructions
    // This allows us to support all WASM instructions without implementing
    // full stack validation logic for every instruction
    Ok(())
}
