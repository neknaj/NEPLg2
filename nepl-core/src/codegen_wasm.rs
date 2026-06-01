//! WASM backend for NEPLG2.

extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ElementMode, ElementSection, ElementSegment, Elements,
    Encode, EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, MemArg, MemorySection, MemoryType, Module, RefType, TableSection, TableType,
    TypeSection, ValType,
};

use crate::backend_scalar_type::BackendScalarType;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::hir::*;
use crate::intrinsic_kinds::{ScalarIntrinsicBackendOp, ScalarIntrinsicKind};
use crate::layout::{
    enum_payload_offset_bytes, intrinsic_storage_type, is_aggregate_storage_type,
    storage_align_bytes, storage_size_bytes, tuple_field_layouts_by_result,
};
use crate::resource_primitives::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
use crate::runtime_helpers::{self, RuntimeHelperKind};
use crate::scalar_primitives::I32ArithmeticPrimitive;
use crate::span::Span;
use crate::typecheck::materialized_callable_symbol_for_link_symbol;
use crate::types::{TypeCtx, TypeId, TypeKind};

mod aggregate;
mod enum_helpers;
mod local_map;
mod string_data;

use aggregate::aggregate_field_layout;
use enum_helpers::{enum_match_type, enum_variant_payload, enum_variant_tag};
use local_map::LocalMap;
use string_data::{lower_strings, StringDataLayout};

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

/// `.neplobj` direct-call fragment が wasm backend の function index 空間へ入れることを
/// 確認した結果。
///
/// この token は「body-missing diagnostic を消してよい」という最終許可ではない。raw body
/// bytes を `CodeSection` へ入れ、relocation を実際に書き換える実装が接続されるまでは、
/// compiler pipeline は従来どおり source fallback を要求する。ここでは次段階のために、
/// function name、signature、relocation target が現在の wasm assembly plan と矛盾しないことだけを
/// backend 側の authority として記録する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplObjWasmDirectCallLinkPlanToken {
    pub materialized_symbol: String,
    pub direct_call_key_hash: u64,
    pub fragment_hash: u64,
    pub assigned_function_index: u32,
    pub resolved_relocations: Vec<NeplObjWasmResolvedDirectCallRelocation>,
    pub backend_feature_set_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplObjWasmResolvedDirectCallRelocation {
    pub byte_offset: u32,
    pub target_symbol: String,
    pub target_function_index: u32,
}

/// `.neplobj` direct-call fragment を wasm function index 空間へ登録できない理由。
///
/// ここで拒否された fragment は backend が消費した artifact ではないため、body-missing
/// diagnostic を抑制してはならない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplObjWasmDirectCallLinkPlanReject {
    UnsupportedBackend { symbol: String },
    BackendFeatureSetMismatch {
        symbol: String,
        expected: u64,
        actual: u64,
    },
    DuplicateFunctionSymbol { symbol: String },
    RelocationTargetMissing { symbol: String, target_symbol: String },
}

type LowerResult<T> = Result<T, Diagnostic>;

fn codegen_error(message: impl Into<String>, span: Span, code: DiagnosticCode) -> Diagnostic {
    Diagnostic::error_with_code(code, message, span)
}

pub fn generate_wasm(ctx: &TypeCtx, module: &HirModule) -> Result<CodegenResult, Vec<Diagnostic>> {
    generate_wasm_with_neplobj_direct_call_fragments(ctx, module, &[])
}

/// `.neplobj` direct-call fragment を同じ wasm module の function body として組み込む codegen API。
///
/// この関数は direct-call MVP 用であり、function value、indirect call、`memo_call` の backend
/// 表現を許可しない。fragment は `plan_neplobj_direct_call_fragments_for_wasm` で function index
/// 空間へ登録できることを確認し、その token が解決した relocation だけを body bytes に patch する。
/// pipeline 側で body-missing diagnostic を消すのは、この API と同じ backend 境界を通る場合に限る。
pub fn generate_wasm_with_neplobj_direct_call_fragments(
    ctx: &TypeCtx,
    module: &HirModule,
    neplobj_direct_call_fragments: &[crate::artifact::NeplObjDirectCallFragmentArtifact],
) -> Result<CodegenResult, Vec<Diagnostic>> {
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
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::ExternSignatureUnsupported,
                )),
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
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::FunctionSignatureUnsupported,
                )),
            )]);
        };
        functions.push(FuncLower::user(f, sig));
    }

    let neplobj_link_tokens = if neplobj_direct_call_fragments.is_empty() {
        Vec::new()
    } else {
        plan_neplobj_direct_call_fragments_for_wasm(ctx, module, neplobj_direct_call_fragments)
            .map_err(|reject| vec![neplobj_link_plan_reject_diagnostic(reject)])?
    };

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
    for token in &neplobj_link_tokens {
        name_to_index.insert(
            token.materialized_symbol.clone(),
            token.assigned_function_index,
        );
    }
    let total_function_slots =
        next_index + functions.len() as u32 + neplobj_link_tokens.len() as u32;

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
    for fragment in neplobj_direct_call_fragments {
        let crate::artifact::NeplObjDirectCallBackendFragment::Wasm(wasm_fragment) =
            fragment.backend();
        let key = (
            neplobj_wasm_val_types_to_encoder(wasm_fragment.params()),
            neplobj_wasm_val_types_to_encoder(wasm_fragment.results()),
        );
        sig_map.entry(key).or_insert_with(|| {
            let idx = type_section.len();
            type_section
                .ty()
                .function(
                    neplobj_wasm_val_types_to_encoder(wasm_fragment.params()),
                    neplobj_wasm_val_types_to_encoder(wasm_fragment.results()),
                );
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
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::LoweredSignatureMissing,
                )),
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
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::LoweredSignatureMissing,
                )),
            )]);
        };
        func_section.function(type_idx);
    }
    for fragment in neplobj_direct_call_fragments {
        let crate::artifact::NeplObjDirectCallBackendFragment::Wasm(wasm_fragment) =
            fragment.backend();
        let key = (
            neplobj_wasm_val_types_to_encoder(wasm_fragment.params()),
            neplobj_wasm_val_types_to_encoder(wasm_fragment.results()),
        );
        let Some(type_idx) = sig_map.get(&key).copied() else {
            return Err(vec![codegen_error(
                format!(
                    "missing lowered wasm signature for .neplobj fragment '{}'",
                    fragment.materialized_symbol()
                ),
                Span::dummy(),
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::LoweredSignatureMissing,
                )),
            )]);
        };
        func_section.function(type_idx);
    }

    let mut code_section = CodeSection::new();
    for f in &functions {
        let body = lower_body(ctx, f, &name_to_index, &sig_map, &strings).map_err(|d| vec![d])?;
        code_section.function(&body);
    }
    for (fragment, token) in neplobj_direct_call_fragments
        .iter()
        .zip(neplobj_link_tokens.iter())
    {
        let body =
            patch_neplobj_wasm_direct_call_body(fragment, token).map_err(|error| {
                vec![neplobj_direct_call_patch_error_diagnostic(
                    fragment.materialized_symbol(),
                    error,
                )]
            })?;
        code_section.raw(&body);
    }

    let mut memory_section = MemorySection::new();
    memory_section.memory(MemoryType {
        minimum: strings.min_pages() as u64,
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
        strings.heap_base().to_le_bytes().to_vec(),
    );
    data_section.active(0, &ConstExpr::i32_const(4), 0u32.to_le_bytes().to_vec());
    for (offset, bytes) in strings.segments() {
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

fn neplobj_link_plan_reject_diagnostic(
    reject: NeplObjWasmDirectCallLinkPlanReject,
) -> Diagnostic {
    let message = match reject {
        NeplObjWasmDirectCallLinkPlanReject::UnsupportedBackend { symbol } => {
            format!(
                ".neplobj direct-call fragment cannot be linked because the current wasm backend does not support the surrounding signature for '{}'",
                symbol
            )
        }
        NeplObjWasmDirectCallLinkPlanReject::BackendFeatureSetMismatch {
            symbol,
            expected,
            actual,
        } => {
            format!(
                ".neplobj direct-call fragment '{}' was built for backend feature set {:016x}, but the current backend expects {:016x}",
                symbol, actual, expected
            )
        }
        NeplObjWasmDirectCallLinkPlanReject::DuplicateFunctionSymbol { symbol } => {
            format!(
                ".neplobj direct-call fragment '{}' conflicts with an existing wasm function symbol",
                symbol
            )
        }
        NeplObjWasmDirectCallLinkPlanReject::RelocationTargetMissing {
            symbol,
            target_symbol,
        } => {
            format!(
                ".neplobj direct-call fragment '{}' references missing direct-call target '{}'",
                symbol, target_symbol
            )
        }
    };
    codegen_error(
        message,
        Span::dummy(),
        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
            crate::diagnostic_codes::WasmDiagnosticCode::NeplObjDirectCallLinkInvalid,
        )),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeplObjWasmDirectCallPatchError {
    RelocationOffsetMissingCallOpcode { byte_offset: u32 },
    RelocationImmediateInvalid { byte_offset: u32 },
}

fn patch_neplobj_wasm_direct_call_body(
    fragment: &crate::artifact::NeplObjDirectCallFragmentArtifact,
    token: &NeplObjWasmDirectCallLinkPlanToken,
) -> Result<Vec<u8>, NeplObjWasmDirectCallPatchError> {
    let crate::artifact::NeplObjDirectCallBackendFragment::Wasm(wasm_fragment) = fragment.backend();
    let mut body = wasm_fragment.function_body_bytes().to_vec();
    for relocation in token.resolved_relocations.iter().rev() {
        patch_wasm_call_index_leb(
            &mut body,
            relocation.byte_offset,
            relocation.target_function_index,
        )?;
    }
    Ok(body)
}

fn patch_wasm_call_index_leb(
    body: &mut Vec<u8>,
    byte_offset: u32,
    target_function_index: u32,
) -> Result<(), NeplObjWasmDirectCallPatchError> {
    let offset = byte_offset as usize;
    if offset == 0 || body.get(offset.wrapping_sub(1)).copied() != Some(0x10) {
        return Err(
            NeplObjWasmDirectCallPatchError::RelocationOffsetMissingCallOpcode { byte_offset },
        );
    }
    let Some(width) = wasm_u32_leb_width_at(body, offset) else {
        return Err(NeplObjWasmDirectCallPatchError::RelocationImmediateInvalid { byte_offset });
    };
    let mut encoded = Vec::new();
    target_function_index.encode(&mut encoded);
    body.splice(offset..offset + width, encoded);
    Ok(())
}

fn wasm_u32_leb_width_at(body: &[u8], offset: usize) -> Option<usize> {
    for width in 0..5 {
        let byte = *body.get(offset + width)?;
        if width == 4 {
            if byte & 0x80 != 0 || byte & 0x70 != 0 {
                return None;
            }
            return Some(width + 1);
        }
        if byte & 0x80 == 0 {
            return Some(width + 1);
        }
    }
    None
}

fn neplobj_direct_call_patch_error_diagnostic(
    symbol: &str,
    error: NeplObjWasmDirectCallPatchError,
) -> Diagnostic {
    let message = match error {
        NeplObjWasmDirectCallPatchError::RelocationOffsetMissingCallOpcode { byte_offset } => {
            format!(
                ".neplobj direct-call fragment '{}' has relocation at byte {} that does not point to a call immediate",
                symbol, byte_offset
            )
        }
        NeplObjWasmDirectCallPatchError::RelocationImmediateInvalid { byte_offset } => {
            format!(
                ".neplobj direct-call fragment '{}' has an invalid call-index LEB128 immediate at byte {}",
                symbol, byte_offset
            )
        }
    };
    codegen_error(
        message,
        Span::dummy(),
        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
            crate::diagnostic_codes::WasmDiagnosticCode::NeplObjDirectCallLinkInvalid,
        )),
    )
}

/// `.neplobj` direct-call fragment を現在の wasm assembly plan に照合する。
///
/// `generate_wasm` は imports と HIR functions から `name_to_index` を作り、その map に基づいて
/// direct call を lowered instruction へ変換する。この関数は同じ index 割当規則に
/// `.neplobj` fragment を追加した場合に、symbol 衝突がなく、relocation target がすべて
/// 解決できることを確認する。raw body bytes の patch / insertion はまだ行わない。
pub fn plan_neplobj_direct_call_fragments_for_wasm(
    ctx: &TypeCtx,
    module: &HirModule,
    fragments: &[crate::artifact::NeplObjDirectCallFragmentArtifact],
) -> Result<Vec<NeplObjWasmDirectCallLinkPlanToken>, NeplObjWasmDirectCallLinkPlanReject> {
    let mut name_to_index = wasm_base_function_index_map(ctx, module)?;
    let mut planned = Vec::new();
    let mut next_index = name_to_index.len() as u32;
    for fragment in fragments {
        let symbol = String::from(fragment.materialized_symbol());
        if name_to_index.contains_key(&symbol) {
            return Err(NeplObjWasmDirectCallLinkPlanReject::DuplicateFunctionSymbol {
                symbol,
            });
        }
        match fragment.backend() {
            crate::artifact::NeplObjDirectCallBackendFragment::Wasm(wasm_fragment) => {
                let expected = crate::artifact::nepl_obj_direct_call_backend_feature_set_hash();
                let actual = fragment.key().backend_feature_set_hash;
                if actual != expected {
                    return Err(
                        NeplObjWasmDirectCallLinkPlanReject::BackendFeatureSetMismatch {
                            symbol,
                            expected,
                            actual,
                        },
                    );
                }
                let _signature = (
                    neplobj_wasm_val_types_to_encoder(wasm_fragment.params()),
                    neplobj_wasm_val_types_to_encoder(wasm_fragment.results()),
                );
            }
        }
        name_to_index.insert(symbol.clone(), next_index);
        planned.push((fragment, symbol, next_index));
        next_index += 1;
    }

    let mut tokens = Vec::new();
    for (fragment, symbol, assigned_function_index) in planned {
        let wasm_fragment = match fragment.backend() {
            crate::artifact::NeplObjDirectCallBackendFragment::Wasm(wasm_fragment) => wasm_fragment,
        };
        let mut resolved_relocations = Vec::new();
        for relocation in wasm_fragment.direct_call_relocations() {
            let target_symbol = materialized_callable_symbol_for_link_symbol(&relocation.target);
            let Some(target_function_index) = name_to_index.get(&target_symbol).copied() else {
                return Err(NeplObjWasmDirectCallLinkPlanReject::RelocationTargetMissing {
                    symbol,
                    target_symbol,
                });
            };
            resolved_relocations.push(NeplObjWasmResolvedDirectCallRelocation {
                byte_offset: relocation.byte_offset,
                target_symbol,
                target_function_index,
            });
        }
        tokens.push(NeplObjWasmDirectCallLinkPlanToken {
            materialized_symbol: symbol,
            direct_call_key_hash: fragment.key().stable_hash,
            fragment_hash: fragment.stable_hash(),
            assigned_function_index,
            resolved_relocations,
            backend_feature_set_hash: fragment.key().backend_feature_set_hash,
        });
    }
    Ok(tokens)
}

fn wasm_base_function_index_map(
    ctx: &TypeCtx,
    module: &HirModule,
) -> Result<BTreeMap<String, u32>, NeplObjWasmDirectCallLinkPlanReject> {
    let mut name_to_index = BTreeMap::new();
    let mut next_index = 0u32;
    for ext in &module.externs {
        if wasm_sig_ids(ctx, ext.result, &ext.params).is_none() {
            return Err(NeplObjWasmDirectCallLinkPlanReject::UnsupportedBackend {
                symbol: ext.local_name.clone(),
            });
        }
        if name_to_index.contains_key(&ext.local_name) {
            return Err(NeplObjWasmDirectCallLinkPlanReject::DuplicateFunctionSymbol {
                symbol: ext.local_name.clone(),
            });
        }
        name_to_index.insert(ext.local_name.clone(), next_index);
        next_index += 1;
    }
    for f in &module.functions {
        if crate::wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f) {
            continue;
        }
        if wasm_sig(ctx, f.result, &f.params).is_none() {
            return Err(NeplObjWasmDirectCallLinkPlanReject::UnsupportedBackend {
                symbol: f.name.clone(),
            });
        }
        if name_to_index.contains_key(&f.name) {
            return Err(NeplObjWasmDirectCallLinkPlanReject::DuplicateFunctionSymbol {
                symbol: f.name.clone(),
            });
        }
        name_to_index.insert(f.name.clone(), next_index);
        next_index += 1;
    }
    Ok(name_to_index)
}

fn neplobj_wasm_val_types_to_encoder(
    values: &[crate::artifact::NeplObjWasmValType],
) -> Vec<ValType> {
    values
        .iter()
        .map(|value| match value {
            crate::artifact::NeplObjWasmValType::I32 => ValType::I32,
            crate::artifact::NeplObjWasmValType::I64 => ValType::I64,
            crate::artifact::NeplObjWasmValType::F32 => ValType::F32,
            crate::artifact::NeplObjWasmValType::F64 => ValType::F64,
        })
        .collect()
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
        TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Char | TypeKind::Str => {
            Some(ValType::I32)
        }
        TypeKind::F32 => Some(ValType::F32),
        TypeKind::Enum { .. } | TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            Some(ValType::I32)
        }
        TypeKind::Reference(_, _) | TypeKind::Box(_) => Some(ValType::I32),
        TypeKind::Function { .. } => Some(ValType::I32),
        TypeKind::Var(_) => Some(ValType::I32),
        TypeKind::Named(_) => match BackendScalarType::from_type_kind(kind) {
            Some(scalar) if scalar.is_wasm_i64() => Some(ValType::I64),
            Some(scalar) if scalar.is_wasm_f64() => Some(ValType::F64),
            Some(_) | None => Some(ValType::I32),
        },
        TypeKind::Apply { .. } => {
            // std::eprintln!("valtype: Apply is Some(I32)");
            Some(ValType::I32)
        }
        _ => None,
    }
}

fn find_alloc_index(name_map: &BTreeMap<String, u32>, current_func: &str) -> Option<u32> {
    runtime_helpers::find_runtime_helper_index(
        name_map,
        RuntimeHelperKind::Alloc,
        Some(current_func),
    )
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
    if let Some(idx) = locals.alloc_helper_idx() {
        insts.push(Instruction::Call(idx));
    } else {
        emit_inline_alloc(locals, insts);
    }
}

fn emit_scalar_store(
    ty_kind: &TypeKind,
    vt: ValType,
    span: Span,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<()> {
    match vt {
        ValType::I32 => {
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
        }
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
                "unsupported scalar address valtype reached wasm codegen",
                span,
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::FieldValueTypeUnsupported,
                )),
            ));
        }
    }
    Ok(())
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
    strings: &StringDataLayout,
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
    strings: &StringDataLayout,
) -> LowerResult<Function> {
    let mut locals = LocalMap::new();
    for p in &func.params {
        locals.register_param(p.name.clone(), valtype(&ctx.get(ctx.resolve_id(p.ty))));
    }
    locals.set_alloc_helper_idx(find_alloc_index(name_map, &func.name));

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
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::ReturnValueMissing,
                    )),
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
                            DiagnosticCode::Backend(
                                crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                    crate::diagnostic_codes::WasmDiagnosticCode::RawLineParseError,
                                ),
                            ),
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
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::LlvmIrBodyUnsupported,
                )),
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
    strings: &StringDataLayout,
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
            let _ = locals.ensure_local(name.clone(), valtype(&ctx.get(value.ty)));
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
            | HirExprKind::MemoizedFunctionValue(_)
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
        FuncRef::Builtin(n) | FuncRef::User(n, _, _) => n.clone(),
        FuncRef::Trait {
            application,
            method,
            self_ty,
        } => {
            let mut s = application.display_name(ctx);
            s.push_str("::");
            s.push_str(method.as_str());
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
        FuncRef::Builtin(n) | FuncRef::User(n, _, _) => name_map.get(n),
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
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                crate::diagnostic_codes::WasmDiagnosticCode::FunctionUnknown,
            )),
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
    strings: &StringDataLayout,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    let mut stack = Vec::new();
    stack.push(SimpleExprFrame::Expr(expr));
    while let Some(frame) = stack.pop() {
        match frame {
            SimpleExprFrame::Expr(expr) => {
                match &expr.kind {
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
                            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::StringLiteralNotFound, )),
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
                            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(crate::diagnostic_codes::WasmDiagnosticCode::VariableUnknown)),
                        ));
                        }
                    }
                    HirExprKind::FnValue(name) | HirExprKind::MemoizedFunctionValue(name) => {
                        if let Some(fidx) = find_function_value_index(name_map, name.symbol()) {
                            insts.push(Instruction::I32Const(fidx as i32));
                        } else {
                            return Err(codegen_error(
                            format!("unknown function value '{}' reached wasm codegen", name),
                            expr.span,
                            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::FunctionValueUnknown, )),
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
                    | HirExprKind::Deref(_) => {
                        unreachable!("iterative wasm lowering precheck failed")
                    }
                }
            }
            SimpleExprFrame::FinishCall { callee, span } => {
                emit_direct_call(ctx, callee, span, name_map, insts)?;
            }
        }
    }
    Ok(valtype(&ctx.get(expr.ty)))
}

fn block_wrapped_if(expr: &HirExpr) -> Option<&HirExpr> {
    let HirExprKind::Block(block) = &expr.kind else {
        return None;
    };
    let [line] = block.lines.as_slice() else {
        return None;
    };
    if line.drop_result {
        return None;
    }
    match &line.expr.kind {
        HirExprKind::If { .. } => Some(&line.expr),
        _ => None,
    }
}

// Lower else-if decision trees without consuming one Rust/WASM call frame per arm.
fn gen_if_else_chain(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringDataLayout,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    let result_ty = valtype(&ctx.get(expr.ty));
    let mut current = expr;
    let mut pending_ends = 0usize;
    loop {
        while let Some(inner) = block_wrapped_if(current) {
            current = inner;
        }

        let HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } = &current.kind
        else {
            gen_expr(ctx, current, name_map, sig_map, strings, locals, insts)?;
            break;
        };

        gen_expr(ctx, cond, name_map, sig_map, strings, locals, insts)?;
        match result_ty {
            Some(vt) => insts.push(Instruction::If(wasm_encoder::BlockType::Result(vt))),
            None => insts.push(Instruction::If(wasm_encoder::BlockType::Empty)),
        }
        pending_ends += 1;
        gen_expr(ctx, then_branch, name_map, sig_map, strings, locals, insts)?;
        insts.push(Instruction::Else);
        current = else_branch;
    }
    for _ in 0..pending_ends {
        insts.push(Instruction::End);
    }
    Ok(result_ty)
}

fn gen_scalar_intrinsic(
    ctx: &TypeCtx,
    kind: ScalarIntrinsicKind,
    args: &[HirExpr],
    span: Span,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringDataLayout,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    let expected = kind.argument_count();
    if args.len() != expected {
        return Err(codegen_error(
            format!(
                "intrinsic {} expects {} arguments",
                kind.intrinsic_name(),
                expected
            ),
            span,
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicArityMismatch,
            )),
        ));
    }

    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
    Ok(match kind.backend_op() {
        ScalarIntrinsicBackendOp::I32ToF32 => {
            insts.push(Instruction::F32ConvertI32S);
            Some(ValType::F32)
        }
        ScalarIntrinsicBackendOp::I32ToU8 => {
            insts.push(Instruction::I32Const(255));
            insts.push(Instruction::I32And);
            Some(ValType::I32)
        }
        ScalarIntrinsicBackendOp::U8ToI32 => Some(ValType::I32),
        ScalarIntrinsicBackendOp::F32ToI32 => {
            insts.push(Instruction::I32TruncF32S);
            Some(ValType::I32)
        }
        ScalarIntrinsicBackendOp::I32Identity => Some(ValType::I32),
        ScalarIntrinsicBackendOp::I64Identity => Some(ValType::I64),
        ScalarIntrinsicBackendOp::ReinterpretI32AsF32 => {
            insts.push(Instruction::F32ReinterpretI32);
            Some(ValType::F32)
        }
        ScalarIntrinsicBackendOp::ReinterpretF32AsI32 => {
            insts.push(Instruction::I32ReinterpretF32);
            Some(ValType::I32)
        }
    })
}

fn gen_i32_arithmetic_intrinsic(
    ctx: &TypeCtx,
    kind: I32ArithmeticPrimitive,
    args: &[HirExpr],
    span: Span,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringDataLayout,
    locals: &mut LocalMap,
    insts: &mut Vec<Instruction<'static>>,
) -> LowerResult<Option<ValType>> {
    let Some(expected) = kind.codegen_argument_count() else {
        return Err(codegen_error(
            "unsupported i32 arithmetic codegen intrinsic",
            span,
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
            )),
        ));
    };
    if args.len() != expected {
        return Err(codegen_error(
            format!(
                "intrinsic {} expects {} arguments",
                kind.codegen_intrinsic_name().unwrap_or("i32_arithmetic"),
                expected
            ),
            span,
            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicArityMismatch,
            )),
        ));
    }

    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
    gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
    match kind {
        I32ArithmeticPrimitive::Add => insts.push(Instruction::I32Add),
        I32ArithmeticPrimitive::Sub | I32ArithmeticPrimitive::Mul => {
            return Err(codegen_error(
                "unsupported i32 arithmetic codegen intrinsic",
                span,
                DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
                )),
            ));
        }
    }
    Ok(Some(ValType::I32))
}

fn gen_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    name_map: &BTreeMap<String, u32>,
    sig_map: &BTreeMap<(Vec<ValType>, Vec<ValType>), u32>,
    strings: &StringDataLayout,
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
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::StringLiteralNotFound,
                    )),
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
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::VariableUnknown,
                    )),
                ));
            }
        }
        HirExprKind::FnValue(name) | HirExprKind::MemoizedFunctionValue(name) => {
            if let Some(fidx) = find_function_value_index(name_map, name.symbol()) {
                insts.push(Instruction::I32Const(fidx as i32));
                Some(ValType::I32)
            } else {
                return Err(codegen_error(
                    format!("unknown function value '{}' reached wasm codegen", name),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::FunctionValueUnknown,
                    )),
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
            ..
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
                        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::IndirectSignatureMissing, )),
                    ));
                }
            } else {
                return Err(codegen_error(
                    "unsupported indirect call signature for wasm",
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::IndirectSignatureUnsupported,
                    )),
                ));
            }
            valtype(&ctx.get(expr.ty))
        }
        HirExprKind::If { .. } => {
            gen_if_else_chain(ctx, expr, name_map, sig_map, strings, locals, insts)?
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
                let size = storage_size_bytes(ctx, ty) as i32;
                insts.push(Instruction::I32Const(size));
                Some(ValType::I32)
            } else if name == "align_of" {
                let ty = type_args[0];
                let align = storage_align_bytes(ctx, ty) as i32;
                insts.push(Instruction::I32Const(align));
                Some(ValType::I32)
            } else if name == "load" {
                let ty = intrinsic_storage_type(ctx, type_args[0], expr.ty);
                let ty_kind = ctx.get(ty);
                if is_aggregate_storage_type(ctx, ty) {
                    let size = storage_size_bytes(ctx, ty) as i32;
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
                let ty = intrinsic_storage_type(ctx, type_args[0], args[1].ty);
                let ty_kind = ctx.get(ty);
                if is_aggregate_storage_type(ctx, ty) {
                    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                    let dst_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(dst_local));
                    gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
                    let src_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(src_local));
                    let size = storage_size_bytes(ctx, ty) as i32;
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
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicArityMismatch,
                            ),
                        ),
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
                        let size = storage_size_bytes(ctx, field_ty) as i32;
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
                        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::FieldSelectorUnsupported, )),
                    ));
                }
                gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?;
                let idx_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalSet(idx_local));
                if is_aggregate_storage_type(ctx, expr.ty) {
                    let size = storage_size_bytes(ctx, expr.ty) as i32;
                    insts.push(Instruction::I32Const(size));
                    emit_alloc_call(locals, insts);
                    let dst_local = locals.alloc_temp(ValType::I32);
                    insts.push(Instruction::LocalSet(dst_local));
                    for layout in candidate_layouts {
                        let position = layout.index;
                        let offset = layout.field.offset;
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
                    for layout in candidate_layouts {
                        let position = layout.index;
                        let field_ty = layout.field.ty;
                        let offset = layout.field.offset;
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
                                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::FieldValueTypeUnsupported, )),
                                ));
                            }
                        }
                        insts.push(Instruction::End);
                    }
                    insts.push(Instruction::LocalGet(out_local));
                    Some(ValType::I32)
                }
            } else if name == "get_field_ref" {
                if args.len() != 2 {
                    return Err(codegen_error(
                        "intrinsic get_field_ref requires two args",
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicArityMismatch,
                            ),
                        ),
                    ));
                }
                let base_ty = match ctx.get(ctx.resolve_id(args[0].ty)) {
                    TypeKind::Reference(inner, _) => inner,
                    _ => args[0].ty,
                };
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(ctx, base_ty, &args[1], strings)
                else {
                    return Err(codegen_error(
                        "unsupported get_field_ref selector reached wasm codegen",
                        expr.span,
                        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::FieldSelectorUnsupported, )),
                    ));
                };
                gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?;
                if offset != 0 {
                    insts.push(Instruction::I32Const(offset as i32));
                    insts.push(Instruction::I32Add);
                }
                Some(ValType::I32)
            } else if name == "set_field" {
                if args.len() != 3 {
                    return Err(codegen_error(
                        "intrinsic set_field requires three args",
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicArityMismatch,
                            ),
                        ),
                    ));
                }
                let Some((_field_ty, offset)) =
                    aggregate_field_layout(ctx, args[0].ty, &args[1], strings)
                else {
                    return Err(codegen_error(
                        "unsupported set_field selector reached wasm codegen",
                        expr.span,
                        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::FieldSelectorUnsupported, )),
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
                    let size = storage_size_bytes(ctx, field_ty) as i32;
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
            } else if let Some(kind) = ScalarIntrinsicKind::from_intrinsic_name(name) {
                gen_scalar_intrinsic(
                    ctx, kind, args, expr.span, name_map, sig_map, strings, locals, insts,
                )?
            } else if let Some(kind) = I32ArithmeticPrimitive::from_codegen_intrinsic_name(name) {
                gen_i32_arithmetic_intrinsic(
                    ctx, kind, args, expr.span, name_map, sig_map, strings, locals, insts,
                )?
            } else if CollectionSlotBorrowPrimitive::from_intrinsic_name(name).is_some() {
                if args.len() != 2 {
                    return Err(codegen_error(
                        format!("invalid collection slot borrow intrinsic '{}'", name),
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
                            ),
                        ),
                    ));
                }
                let Some(ValType::I32) =
                    gen_expr(ctx, &args[0], name_map, sig_map, strings, locals, insts)?
                else {
                    return Err(codegen_error(
                        "collection slot borrow target must lower to an address",
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
                            ),
                        ),
                    ));
                };
                insts.push(Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                let Some(ValType::I32) =
                    gen_expr(ctx, &args[1], name_map, sig_map, strings, locals, insts)?
                else {
                    return Err(codegen_error(
                        "collection slot borrow offset must lower to i32",
                        expr.span,
                        DiagnosticCode::Backend(
                            crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                                crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
                            ),
                        ),
                    ));
                };
                insts.push(Instruction::I32Add);
                Some(ValType::I32)
            } else if CollectionSlotLifecyclePrimitive::from_intrinsic_name(name).is_some() {
                for arg in args {
                    if gen_expr(ctx, arg, name_map, sig_map, strings, locals, insts)?.is_some() {
                        insts.push(Instruction::Drop);
                    }
                }
                None
            } else if name == "unreachable" {
                insts.push(Instruction::Unreachable);
                None
            } else {
                return Err(codegen_error(
                    format!("unknown intrinsic '{}' reached wasm codegen", name),
                    expr.span,
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::IntrinsicUnknown,
                    )),
                ));
            }
        }
        HirExprKind::EnumConstruct {
            name: _,
            variant,
            payload,
            type_args: _,
        } => {
            let payload_offset = enum_payload_offset_bytes() as i32;
            let payload_storage_size = payload
                .as_ref()
                .map(|p| payload_offset + storage_size_bytes(ctx, p.ty) as i32)
                .unwrap_or(payload_offset);
            let total_size = (storage_size_bytes(ctx, expr.ty) as i32).max(payload_storage_size);
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
                    let payload_size = storage_size_bytes(ctx, p.ty) as i32;
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
                                        DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::EnumPayloadTypeUnsupported, )),
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
                size += storage_size_bytes(ctx, f.ty) as u32;
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
                    let field_size = storage_size_bytes(ctx, field_ty) as i32;
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
                                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::StructFieldTypeUnsupported, )),
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
                size += storage_size_bytes(ctx, item.ty) as u32;
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
                    let item_size = storage_size_bytes(ctx, item_ty) as i32;
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
                                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::TupleElementTypeUnsupported, )),
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
            let enum_match_ty = enum_match_type(ctx, scrutinee.ty);
            let is_enum_match = enum_match_ty.is_some()
                && arms.iter().all(|arm| {
                    matches!(
                        arm.pattern,
                        HirMatchPattern::Variant(_) | HirMatchPattern::Wildcard
                    )
                });
            gen_expr(ctx, scrutinee, name_map, sig_map, strings, locals, insts)?;
            let scrut_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalSet(scrut_local));
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

            if is_enum_match {
                let tag_local = locals.alloc_temp(ValType::I32);
                insts.push(Instruction::LocalGet(scrut_local));
                insts.push(Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                insts.push(Instruction::LocalSet(tag_local));

                let mut pending_ifs = 0usize;
                let mut saw_wildcard = false;
                for arm in arms {
                    let variant = if let HirMatchPattern::Variant(v) = &arm.pattern {
                        let tag = enum_variant_tag(ctx, enum_match_ty.unwrap(), v.as_str());
                        insts.push(Instruction::LocalGet(tag_local));
                        insts.push(Instruction::I32Const(tag as i32));
                        insts.push(Instruction::I32Eq);
                        insts.push(Instruction::If(match result_ty {
                            Some(vt) => wasm_encoder::BlockType::Result(vt),
                            None => wasm_encoder::BlockType::Empty,
                        }));
                        pending_ifs += 1;
                        Some(v.as_str())
                    } else {
                        saw_wildcard = true;
                        None
                    };
                    locals.begin_scope();
                    if let (Some(variant), Some(bind)) = (variant, &arm.bind_local) {
                        if let Some(payload_ty) =
                            enum_variant_payload(ctx, enum_match_ty.unwrap(), variant)
                        {
                            let bind_ty = arm.bind_ty.unwrap_or(payload_ty);
                            let lidx =
                                locals.ensure_local(bind.clone(), valtype(&ctx.get(bind_ty)));
                            let payload_offset = enum_payload_offset_bytes() as i32;
                            if matches!(arm.bind_mode, Some(HirMatchBindMode::Borrowed { .. })) {
                                emit_linear_addr_from_local(scrut_local, payload_offset, insts);
                                insts.push(Instruction::LocalSet(lidx));
                            } else if is_aggregate_storage_type(ctx, payload_ty) {
                                let payload_size = storage_size_bytes(ctx, payload_ty) as i32;
                                insts.push(Instruction::I32Const(payload_size));
                                emit_alloc_call(locals, insts);
                                let dst_local = locals.alloc_temp(ValType::I32);
                                insts.push(Instruction::LocalSet(dst_local));
                                emit_copy_linear_bytes(
                                    dst_local,
                                    0,
                                    scrut_local,
                                    payload_offset,
                                    payload_size,
                                    insts,
                                );
                                insts.push(Instruction::LocalGet(dst_local));
                                insts.push(Instruction::LocalSet(lidx));
                            } else if let Some(vt) = valtype(&ctx.get(payload_ty)) {
                                emit_linear_addr_from_local(scrut_local, payload_offset, insts);
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
                                            DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm( crate::diagnostic_codes::WasmDiagnosticCode::EnumPayloadTypeUnsupported, )),
                                        ));
                                    }
                                }
                                insts.push(Instruction::LocalSet(lidx));
                            }
                        }
                    }
                    gen_expr(ctx, &arm.body, name_map, sig_map, strings, locals, insts)?;
                    locals.end_scope();
                    if !matches!(&arm.pattern, HirMatchPattern::Wildcard) {
                        insts.push(Instruction::Else);
                    }
                }

                if !saw_wildcard {
                    insts.push(Instruction::Unreachable);
                }
                for _ in 0..pending_ifs {
                    insts.push(Instruction::End);
                }
            } else {
                let mut pending_ifs = 0usize;
                let mut saw_wildcard = false;
                for arm in arms {
                    match &arm.pattern {
                        HirMatchPattern::IntLiteral(v) => {
                            insts.push(Instruction::LocalGet(scrut_local));
                            insts.push(Instruction::I32Const(*v));
                            insts.push(Instruction::I32Eq);
                            insts.push(Instruction::If(match result_ty {
                                Some(vt) => wasm_encoder::BlockType::Result(vt),
                                None => wasm_encoder::BlockType::Empty,
                            }));
                            pending_ifs += 1;
                        }
                        HirMatchPattern::BoolLiteral(v) => {
                            insts.push(Instruction::LocalGet(scrut_local));
                            insts.push(Instruction::I32Const(if *v { 1 } else { 0 }));
                            insts.push(Instruction::I32Eq);
                            insts.push(Instruction::If(match result_ty {
                                Some(vt) => wasm_encoder::BlockType::Result(vt),
                                None => wasm_encoder::BlockType::Empty,
                            }));
                            pending_ifs += 1;
                        }
                        HirMatchPattern::Wildcard => {
                            saw_wildcard = true;
                        }
                        HirMatchPattern::Variant(_) => unreachable!(),
                    }
                    locals.begin_scope();
                    gen_expr(ctx, &arm.body, name_map, sig_map, strings, locals, insts)?;
                    locals.end_scope();
                    if !matches!(&arm.pattern, HirMatchPattern::Wildcard) {
                        insts.push(Instruction::Else);
                    }
                }
                if !saw_wildcard {
                    insts.push(Instruction::Unreachable);
                }
                for _ in 0..pending_ifs {
                    insts.push(Instruction::End);
                }
            }
            insts.push(Instruction::End);
            result_ty
        }
        HirExprKind::Let { name, value, .. } => {
            let idx = locals.ensure_local(name.clone(), valtype(&ctx.get(value.ty)));
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
                    DiagnosticCode::Backend(crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                        crate::diagnostic_codes::WasmDiagnosticCode::VariableUnknown,
                    )),
                ));
            }
            None
        }
        HirExprKind::Drop { .. } => {
            // For now, Drop is a no-op at the wasm level.
            None
        }
        HirExprKind::AddrOf(inner) => {
            let inner_ty = ctx.resolve_id(inner.ty);
            if is_aggregate_storage_type(ctx, inner_ty) {
                gen_expr(ctx, inner, name_map, sig_map, strings, locals, insts)?;
                return Ok(Some(ValType::I32));
            }

            let produced = gen_expr(ctx, inner, name_map, sig_map, strings, locals, insts)?;
            let Some(vt) = valtype(&ctx.get(inner_ty)) else {
                if produced.is_some() {
                    insts.push(Instruction::Drop);
                }
                insts.push(Instruction::I32Const(0));
                return Ok(Some(ValType::I32));
            };

            let value_local = locals.alloc_temp(vt);
            insts.push(Instruction::LocalSet(value_local));
            insts.push(Instruction::I32Const(
                storage_size_bytes(ctx, inner_ty) as i32
            ));
            emit_alloc_call(locals, insts);
            let ptr_local = locals.alloc_temp(ValType::I32);
            insts.push(Instruction::LocalSet(ptr_local));
            insts.push(Instruction::LocalGet(ptr_local));
            insts.push(Instruction::LocalGet(value_local));
            emit_scalar_store(&ctx.get(inner_ty), vt, expr.span, insts)?;
            insts.push(Instruction::LocalGet(ptr_local));
            Some(ValType::I32)
        }
        HirExprKind::Deref(inner) => {
            let ty = ctx.resolve_id(expr.ty);
            if is_aggregate_storage_type(ctx, ty) {
                let size = storage_size_bytes(ctx, ty) as i32;
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
// Minimal wasm text parser for #wasm blocks
// ---------------------------------------------------------------------

fn parse_wasm_line(line: &str, locals: &LocalMap) -> Result<Vec<Instruction<'static>>, String> {
    crate::wasm_shared::parse_wasm_line_with_lookup(line, |name| locals.lookup(name))
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;

    use super::{
        generate_wasm_with_neplobj_direct_call_fragments,
        plan_neplobj_direct_call_fragments_for_wasm, NeplObjWasmDirectCallLinkPlanReject,
    };
    use crate::artifact::{
        nepl_obj_empty_generic_instantiation_hash, NeplObjDirectCallBackendFragment,
        NeplObjDirectCallFragmentArtifact, NeplObjDirectCallKey, NeplObjWasmDirectCallFragment,
        NeplObjWasmDirectCallRelocation, NeplObjWasmValType,
    };
    use crate::ast::Effect;
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::hir::{
        FuncRef, HirBlock, HirBody, HirExtern, HirExpr, HirExprKind, HirFunction, HirLine,
        HirModule, HirParam,
    };
    use crate::span::Span;
    use crate::typecheck::{materialized_callable_symbol_for_link_symbol, PublicCallableLinkSymbol};
    use crate::types::{TypeCtx, TypeId};

    fn link_symbol(name: &str, signature_hash: u64) -> PublicCallableLinkSymbol {
        PublicCallableLinkSymbol {
            source_path: String::from("/stdlib/core/math.nepl"),
            name: String::from(name),
            signature_hash,
        }
    }

    fn direct_call_key(symbol: PublicCallableLinkSymbol) -> NeplObjDirectCallKey {
        NeplObjDirectCallKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            symbol,
            0x20,
            0x30,
            nepl_obj_empty_generic_instantiation_hash(),
            0x40,
            0x50,
        )
    }

    fn direct_call_fragment(
        name: &str,
        signature_hash: u64,
        relocations: Vec<NeplObjWasmDirectCallRelocation>,
    ) -> NeplObjDirectCallFragmentArtifact {
        let function_body_bytes = if relocations.is_empty() {
            Vec::from([0x00, 0x41, 0x00, 0x0b])
        } else {
            Vec::from([0x00, 0x10, 0x00, 0x0b])
        };
        direct_call_fragment_with_body(name, signature_hash, function_body_bytes, relocations)
    }

    fn direct_call_fragment_with_body(
        name: &str,
        signature_hash: u64,
        function_body_bytes: Vec<u8>,
        relocations: Vec<NeplObjWasmDirectCallRelocation>,
    ) -> NeplObjDirectCallFragmentArtifact {
        NeplObjDirectCallFragmentArtifact::new(
            direct_call_key(link_symbol(name, signature_hash)),
            NeplObjDirectCallBackendFragment::Wasm(
                NeplObjWasmDirectCallFragment::new(
                    Vec::new(),
                    Vec::from([NeplObjWasmValType::I32]),
                    function_body_bytes,
                    relocations,
                )
                .expect("test fixture uses a valid wasm direct-call fragment"),
            ),
        )
    }

    fn empty_i32_module(types: &mut TypeCtx) -> (HirModule, TypeId) {
        let i32_ty = types.i32();
        let fn_ty = types.function(Vec::new(), Vec::new(), i32_ty, Effect::Pure);
        (
            HirModule {
                functions: Vec::from([HirFunction {
                    doc: None,
                    name: String::from("main"),
                    origin_name: String::from("main"),
                    func_ty: fn_ty,
                    params: Vec::new(),
                    result: i32_ty,
                    effect: Effect::Pure,
                    body: HirBody::Block(HirBlock {
                        lines: Vec::from([HirLine {
                            expr: crate::hir::HirExpr {
                                ty: i32_ty,
                                kind: crate::hir::HirExprKind::LiteralI32(0),
                                span: Span::dummy(),
                            },
                            drop_result: false,
                        }]),
                        ty: i32_ty,
                        span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                }]),
                entry: Some(String::from("main")),
                externs: Vec::new(),
                string_literals: Vec::new(),
                traits: Vec::new(),
                impls: Vec::new(),
            },
            i32_ty,
        )
    }

    fn module_calling_materialized_i32(
        types: &mut TypeCtx,
        callee_symbol: String,
    ) -> (HirModule, TypeId) {
        let i32_ty = types.i32();
        let fn_ty = types.function(Vec::new(), Vec::new(), i32_ty, Effect::Pure);
        (
            HirModule {
                functions: Vec::from([HirFunction {
                    doc: None,
                    name: String::from("main"),
                    origin_name: String::from("main"),
                    func_ty: fn_ty,
                    params: Vec::new(),
                    result: i32_ty,
                    effect: Effect::Pure,
                    body: HirBody::Block(HirBlock {
                        lines: Vec::from([HirLine {
                            expr: HirExpr {
                                ty: i32_ty,
                                kind: HirExprKind::Call {
                                    callee: FuncRef::User(callee_symbol, Vec::new(), None),
                                    args: Vec::new(),
                                },
                                span: Span::dummy(),
                            },
                            drop_result: false,
                        }]),
                        ty: i32_ty,
                        span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                }]),
                entry: Some(String::from("main")),
                externs: Vec::new(),
                string_literals: Vec::new(),
                traits: Vec::new(),
                impls: Vec::new(),
            },
            i32_ty,
        )
    }

    fn append_dummy_i32_functions(types: &mut TypeCtx, module: &mut HirModule, count: usize) {
        let i32_ty = types.i32();
        for index in 0..count {
            let fn_ty = types.function(Vec::new(), Vec::new(), i32_ty, Effect::Pure);
            module.functions.push(HirFunction {
                doc: None,
                name: format!("dummy_{index}"),
                origin_name: format!("dummy_{index}"),
                func_ty: fn_ty,
                params: Vec::new(),
                result: i32_ty,
                effect: Effect::Pure,
                body: HirBody::Block(HirBlock {
                    lines: Vec::from([HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::LiteralI32(0),
                            span: Span::dummy(),
                        },
                        drop_result: false,
                    }]),
                    ty: i32_ty,
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            });
        }
    }

    fn call_exported_main_i32(wasm: &[u8]) -> i32 {
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, wasm).expect("generated wasm should validate");
        let linker = wasmi::Linker::new(&engine);
        let mut store = wasmi::Store::new(&engine, ());
        let instance = linker
            .instantiate(&mut store, &module)
            .expect("module should instantiate")
            .start(&mut store)
            .expect("module start should complete");
        let main = instance
            .get_typed_func::<(), i32>(&store, "main")
            .expect("main export should have unit -> i32 signature");
        main.call(&mut store, ())
            .expect("main call should execute successfully")
    }

    /// fragment payload が wasm function index 空間へ登録できる場合だけ token を作る。
    /// relocation target は、同じ plan 内で後から追加される fragment も含めて解決する。
    #[test]
    fn neplobj_wasm_link_plan_resolves_fragment_relocations() {
        let mut types = TypeCtx::new();
        let (module, _) = empty_i32_module(&mut types);
        let target = link_symbol("dep_target", 0x22);
        let target_symbol = materialized_callable_symbol_for_link_symbol(&target);
        let caller = direct_call_fragment(
            "dep_caller",
            0x21,
            Vec::from([NeplObjWasmDirectCallRelocation {
                byte_offset: 2,
                target,
            }]),
        );
        let callee = direct_call_fragment("dep_target", 0x22, Vec::new());

        let tokens =
            plan_neplobj_direct_call_fragments_for_wasm(&types, &module, &[caller, callee])
                .expect("fragments should fit the current wasm assembly plan");

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].assigned_function_index, 1);
        assert_eq!(tokens[1].assigned_function_index, 2);
        assert_eq!(tokens[0].resolved_relocations.len(), 1);
        assert_eq!(tokens[0].resolved_relocations[0].target_symbol, target_symbol);
        assert_eq!(tokens[0].resolved_relocations[0].target_function_index, 2);
    }

    /// HIR / import が既に持つ function symbol と fragment symbol が衝突する場合、その payload は
    /// backend が消費できる artifact ではない。diagnostic 抑制へ進まず fail-closed に拒否する。
    #[test]
    fn neplobj_wasm_link_plan_rejects_duplicate_function_symbol() {
        let mut types = TypeCtx::new();
        let (mut module, i32_ty) = empty_i32_module(&mut types);
        let duplicate_symbol = link_symbol("dep_value", 0x21);
        module.functions[0].name = materialized_callable_symbol_for_link_symbol(&duplicate_symbol);
        let fragment = direct_call_fragment("dep_value", 0x21, Vec::new());

        let result = plan_neplobj_direct_call_fragments_for_wasm(&types, &module, &[fragment]);

        assert_eq!(
            result,
            Err(NeplObjWasmDirectCallLinkPlanReject::DuplicateFunctionSymbol {
                symbol: module.functions[0].name.clone(),
            })
        );
        assert_eq!(module.functions[0].result, i32_ty);
    }

    /// relocation target を現在の function index 空間で解決できない fragment は、body bytes を
    /// code section へ入れる前に拒否する。古い index を含む object をそのまま使わないための境界である。
    #[test]
    fn neplobj_wasm_link_plan_rejects_missing_relocation_target() {
        let mut types = TypeCtx::new();
        let (module, _) = empty_i32_module(&mut types);
        let missing = link_symbol("missing_dep", 0x99);
        let missing_symbol = materialized_callable_symbol_for_link_symbol(&missing);
        let fragment = direct_call_fragment(
            "dep_caller",
            0x21,
            Vec::from([NeplObjWasmDirectCallRelocation {
                byte_offset: 2,
                target: missing,
            }]),
        );
        let caller_symbol = String::from(fragment.materialized_symbol());

        let result = plan_neplobj_direct_call_fragments_for_wasm(&types, &module, &[fragment]);

        assert_eq!(
            result,
            Err(NeplObjWasmDirectCallLinkPlanReject::RelocationTargetMissing {
                symbol: caller_symbol,
                target_symbol: missing_symbol,
            })
        );
    }

    /// 実際の `generate_wasm` が拒否する signature を、`.neplobj` link-plan だけが許可してはならない。
    /// extern の signature も同じ index 空間の一部なので、fragment を登録する前に fail-closed に止める。
    #[test]
    fn neplobj_wasm_link_plan_rejects_unsupported_extern_signature() {
        let mut types = TypeCtx::new();
        let (mut module, _) = empty_i32_module(&mut types);
        let never_ty = types.never();
        module.externs.push(HirExtern {
            module: String::from("env"),
            name: String::from("bad"),
            local_name: String::from("bad_extern"),
            params: Vec::from([never_ty]),
            result: types.unit(),
            effect: Effect::Impure,
            span: Span::dummy(),
        });
        let fragment = direct_call_fragment("dep_value", 0x21, Vec::new());

        let result = plan_neplobj_direct_call_fragments_for_wasm(&types, &module, &[fragment]);

        assert_eq!(
            result,
            Err(NeplObjWasmDirectCallLinkPlanReject::UnsupportedBackend {
                symbol: String::from("bad_extern"),
            })
        );
    }

    /// HIR function の signature 検査も、`generate_wasm` と link-plan token で同じ境界にする。
    /// token だけ成功すると後続の body insertion が実 module と異なる index map を前提にしてしまう。
    #[test]
    fn neplobj_wasm_link_plan_rejects_unsupported_function_signature() {
        let mut types = TypeCtx::new();
        let (mut module, _) = empty_i32_module(&mut types);
        let never_ty = types.never();
        let i32_ty = types.i32();
        module.functions[0].params = Vec::from([HirParam {
            name: String::from("never_value"),
            ty: never_ty,
            mutable: false,
        }]);
        module.functions[0].func_ty =
            types.function(Vec::new(), Vec::from([never_ty]), i32_ty, Effect::Pure);
        let fragment = direct_call_fragment("dep_value", 0x21, Vec::new());

        let result = plan_neplobj_direct_call_fragments_for_wasm(&types, &module, &[fragment]);

        assert_eq!(
            result,
            Err(NeplObjWasmDirectCallLinkPlanReject::UnsupportedBackend {
                symbol: String::from("main"),
            })
        );
    }

    /// link-plan token と body insertion を同じ backend 境界で通せる場合だけ、
    /// materialized direct call は実 wasm function として呼び出せる。
    #[test]
    fn neplobj_wasm_codegen_inserts_direct_call_fragment_body() {
        let mut types = TypeCtx::new();
        let caller_link_symbol = link_symbol("dep_value", 0x21);
        let caller_symbol = materialized_callable_symbol_for_link_symbol(&caller_link_symbol);
        let (module, _) = module_calling_materialized_i32(&mut types, caller_symbol);
        let fragment = direct_call_fragment_with_body(
            "dep_value",
            0x21,
            Vec::from([0x00, 0x41, 0x07, 0x0b]),
            Vec::new(),
        );

        let codegen = generate_wasm_with_neplobj_direct_call_fragments(&types, &module, &[fragment])
            .expect("direct-call fragment should be inserted into wasm codegen");
        let wasm = codegen.bytes.expect("wasm bytes should be generated");

        assert_eq!(call_exported_main_i32(&wasm), 7);
    }

    /// direct-call relocation は final module の function index で patch する。
    /// ここでは index が 127 を超える配置にし、LEB128 immediate の長さが変わる場合も検証する。
    #[test]
    fn neplobj_wasm_codegen_patches_fragment_direct_call_relocation() {
        let mut types = TypeCtx::new();
        let caller_link_symbol = link_symbol("dep_caller", 0x21);
        let caller_symbol = materialized_callable_symbol_for_link_symbol(&caller_link_symbol);
        let target_link_symbol = link_symbol("dep_target", 0x22);
        let (mut module, _) = module_calling_materialized_i32(&mut types, caller_symbol);
        append_dummy_i32_functions(&mut types, &mut module, 130);
        let caller = direct_call_fragment_with_body(
            "dep_caller",
            0x21,
            Vec::from([0x00, 0x10, 0x00, 0x0b]),
            Vec::from([NeplObjWasmDirectCallRelocation {
                byte_offset: 2,
                target: target_link_symbol,
            }]),
        );
        let target = direct_call_fragment_with_body(
            "dep_target",
            0x22,
            Vec::from([0x00, 0x41, 0x09, 0x0b]),
            Vec::new(),
        );

        let codegen =
            generate_wasm_with_neplobj_direct_call_fragments(&types, &module, &[caller, target])
                .expect("fragment relocation should be patched during wasm codegen");
        let wasm = codegen.bytes.expect("wasm bytes should be generated");

        assert_eq!(call_exported_main_i32(&wasm), 9);
    }

    /// relocation offset が call immediate を指していても、既存 immediate 自体が壊れている
    /// fragment は拒否する。cache payload は backend feature set だけでなく body encoding も
    /// 現在の patcher が理解できる形でなければならない。
    #[test]
    fn neplobj_wasm_codegen_rejects_invalid_relocation_immediate() {
        let mut types = TypeCtx::new();
        let caller_link_symbol = link_symbol("dep_caller", 0x21);
        let caller_symbol = materialized_callable_symbol_for_link_symbol(&caller_link_symbol);
        let target_link_symbol = link_symbol("dep_target", 0x22);
        let (module, _) = module_calling_materialized_i32(&mut types, caller_symbol);
        let caller = direct_call_fragment_with_body(
            "dep_caller",
            0x21,
            Vec::from([0x00, 0x10, 0x80, 0x80, 0x80, 0x80, 0x80, 0x0b]),
            Vec::from([NeplObjWasmDirectCallRelocation {
                byte_offset: 2,
                target: target_link_symbol,
            }]),
        );
        let target = direct_call_fragment_with_body(
            "dep_target",
            0x22,
            Vec::from([0x00, 0x41, 0x09, 0x0b]),
            Vec::new(),
        );

        let result =
            generate_wasm_with_neplobj_direct_call_fragments(&types, &module, &[caller, target]);
        let Err(diagnostics) = result else {
            panic!("invalid relocation immediate must be rejected");
        };

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code
            == crate::diagnostic_codes::DiagnosticCode::Backend(
                crate::diagnostic_codes::BackendDiagnosticCode::Wasm(
                    crate::diagnostic_codes::WasmDiagnosticCode::NeplObjDirectCallLinkInvalid,
                ),
            )));
    }
}
