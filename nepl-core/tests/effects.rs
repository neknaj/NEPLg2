use nepl_core::ast::Effect;
use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::DiagnosticCode;
use nepl_core::effects::{
    external_io_op_from_name, internal_effect_surface_fold, intrinsic_effect, nondet_op_from_name,
    raw_body_direct_callee_effects, raw_body_memory_operations, raw_callee_internal_effect,
    raw_memory_callee_internal_effect, raw_memory_intrinsic_op_from_name, raw_memory_op_from_name,
    ExternalIoOp, InternalEffect, LlvmRawBodyIntrinsic, LlvmRawBodyMemoryOp, NondetOp,
    RawBodyBackendIntrinsic, RawBodyDirectCallee, RawBodyMemoryOp, RawMemoryHelper, RawMemoryOp,
    WasmRawBodyMemoryOp,
};
use nepl_core::error::CoreError;
use nepl_core::hir::HirBody;
use nepl_core::loader::Loader;
use nepl_core::source_map::SourceMap;
use nepl_core::span::{FileId, Span};
use nepl_core::{
    ast::{LlvmIrBlock, WasmBlock},
    check_module, check_module_with_source_map, compile_wasm, lexer, parser, CompileOptions,
    CompileTarget,
};

fn options(target: CompileTarget) -> CompileOptions {
    CompileOptions {
        target: Some(target),
        verbose: false,
        profile: None,
    }
}

fn parse_module(src: &str) -> nepl_core::ast::Module {
    parse_module_with_file_id(FileId(0), src)
}

fn parse_module_with_file_id(file_id: FileId, src: &str) -> nepl_core::ast::Module {
    let lex = lexer::lex(file_id, src);
    let parse = parser::parse_tokens(file_id, lex);
    assert!(
        parse
            .diagnostics
            .iter()
            .all(|d| !matches!(d.severity, Severity::Error)),
        "unexpected parse diagnostics: {:?}",
        parse.diagnostics
    );
    parse.module.expect("module")
}

fn check_source(src: &str, target: CompileTarget) -> Result<(), CoreError> {
    check_module(parse_module(src), options(target))
}

fn check_source_with_path(src: &str, path: &str, target: CompileTarget) -> Result<(), CoreError> {
    let mut source_map = SourceMap::new();
    let file_id = source_map.add(path, String::from(src));
    let module = parse_module_with_file_id(file_id, src);
    check_module_with_source_map(module, Some(&source_map), options(target))
}

fn check_source_as_core_mem_boundary(
    src: &str,
    path: &str,
    target: CompileTarget,
) -> Result<(), CoreError> {
    let path = std::path::PathBuf::from(path);
    let stdlib_root = path
        .parent()
        .and_then(|parent| parent.parent())
        .expect("raw memory boundary fixture must live under a stdlib subdirectory")
        .to_path_buf();
    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load_inline(path, String::from(src)).expect("load");
    check_module_with_source_map(loaded.module, Some(&loaded.source_map), options(target))
}

fn check_source_with_canonical_mem_types(
    src: &str,
    relative_path: &str,
    target: CompileTarget,
) -> Result<(), CoreError> {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    let core_mem = stdlib_root.join("core").join("mem");
    std::fs::create_dir_all(&core_mem).expect("create canonical core/mem dir");
    std::fs::write(
        core_mem.join("types.nepl"),
        r#"
#indent 4
#target wasm

pub struct MemPtr<.T>:
    raw <i32>

pub struct RegionToken<.T>:
    raw <i32>
    size <i32>
"#,
    )
    .expect("write canonical mem types");

    let entry = stdlib_root.join(relative_path);
    if let Some(parent) = entry.parent() {
        std::fs::create_dir_all(parent).expect("create stdlib entry parent");
    }
    std::fs::write(&entry, src).expect("write stdlib entry source");

    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load(&entry).expect("load stdlib entry");
    check_module_with_source_map(loaded.module, Some(&loaded.source_map), options(target))
}

fn assert_has_diag(result: Result<(), CoreError>, code: DiagnosticCode) {
    match result {
        Err(CoreError::Diagnostics(diags)) => assert!(
            diags.iter().any(|d| d.code == code),
            "expected diagnostic {:?}, got {:?}",
            code,
            diags
        ),
        other => panic!("expected diagnostics, got {:?}", other),
    }
}

fn source_file_id_for_suffix(source_map: &SourceMap, suffix: &str) -> FileId {
    source_map
        .iter_paths()
        .find_map(|(file_id, path)| {
            let normalized = path.as_str().replace('\\', "/");
            normalized.ends_with(suffix).then_some(file_id)
        })
        .unwrap_or_else(|| panic!("source suffix not loaded: {suffix}"))
}

fn source_capability_probe_span(file_id: FileId) -> Span {
    Span::new(file_id, 0, 8)
}

#[test]
fn internal_effect_classifies_raw_memory_and_surface_fold() {
    let alloc = raw_callee_internal_effect("alloc_raw").expect("alloc effect");
    assert!(matches!(
        alloc,
        InternalEffect::InternalAlloc { operation } if operation == RawMemoryOp::Alloc
    ));
    assert_eq!(internal_effect_surface_fold(&alloc), Some(Effect::Pure));

    let store = raw_callee_internal_effect("store_i32__i32_i32__unit__pure").expect("store effect");
    assert!(matches!(
        store,
        InternalEffect::UnsafeMemory { operation } if operation == RawMemoryOp::Store
    ));
    assert_eq!(internal_effect_surface_fold(&store), None);
    assert_eq!(intrinsic_effect("load"), Effect::Impure);
    assert_eq!(
        raw_memory_intrinsic_op_from_name("load"),
        Some(RawMemoryOp::Load)
    );
    assert_eq!(
        raw_memory_intrinsic_op_from_name("store"),
        Some(RawMemoryOp::Store)
    );
    assert_eq!(raw_memory_intrinsic_op_from_name("load_i32"), None);

    let io = raw_callee_internal_effect("fd_write").expect("io effect");
    assert!(matches!(
        io,
        InternalEffect::ExternalIo { operation } if operation == ExternalIoOp::FdWrite
    ));
    assert_eq!(raw_memory_callee_internal_effect("fd_write"), None);
    assert_eq!(internal_effect_surface_fold(&io), Some(Effect::Impure));

    let nondet = raw_callee_internal_effect("random_get").expect("nondet effect");
    assert!(matches!(
        nondet,
        InternalEffect::Nondet { operation } if operation == NondetOp::RandomGet
    ));
    assert_eq!(internal_effect_surface_fold(&nondet), Some(Effect::Impure));
}

#[test]
fn host_effect_operation_domains_round_trip_through_typed_classifiers() {
    for &operation in ExternalIoOp::ALL {
        assert_eq!(
            external_io_op_from_name(operation.as_str()),
            Some(operation),
            "external IO operation '{}' must map back to ExternalIoOp",
            operation
        );
        assert_eq!(nondet_op_from_name(operation.as_str()), None);
    }

    for &operation in NondetOp::ALL {
        assert_eq!(
            nondet_op_from_name(operation.as_str()),
            Some(operation),
            "nondeterministic operation '{}' must map back to NondetOp",
            operation
        );
        assert_eq!(external_io_op_from_name(operation.as_str()), None);
    }
}

#[test]
fn raw_memory_helper_domain_round_trips_through_typed_classifier() {
    for &helper in RawMemoryHelper::ALL {
        assert_eq!(
            RawMemoryHelper::from_name(helper.base_name()),
            Some(helper),
            "raw memory helper '{}' must map back to RawMemoryHelper",
            helper
        );
        assert_eq!(
            raw_memory_op_from_name(helper.base_name()),
            Some(helper.operation()),
            "raw memory helper '{}' must map to its RawMemoryOp",
            helper
        );
    }

    for (intrinsic, expected) in [("load", RawMemoryOp::Load), ("store", RawMemoryOp::Store)] {
        assert_eq!(raw_memory_intrinsic_op_from_name(intrinsic), Some(expected));
    }
}

#[test]
fn raw_body_memory_operations_are_typed_by_backend() {
    let wasm = HirBody::Wasm(WasmBlock {
        lines: vec![
            String::from("i32.load"),
            String::from("i64.store"),
            String::from("i32.reload"),
            String::from("custom.loadx"),
            String::from("memory.grow"),
            String::from("memory.copy"),
            String::from("data.drop"),
        ],
        span: Span::dummy(),
    });
    assert_eq!(
        raw_body_memory_operations(&wasm),
        vec![
            RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::Load),
            RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::Store),
            RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryGrow),
            RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::MemoryCopy),
            RawBodyMemoryOp::Wasm(WasmRawBodyMemoryOp::DataDrop),
        ]
    );

    let llvm = HirBody::LlvmIr(LlvmIrBlock {
        lines: vec![
            String::from("%p = alloca i32"),
            String::from("%v = load i32, ptr %p"),
            String::from("store i32 1, ptr %p"),
            String::from("fence seq_cst"),
            String::from("call void @llvm.memcpy_like(ptr %p, ptr %q, i64 4, i1 false)"),
            String::from("call void @llvm.memcpy.p0.p0.i64(ptr %p, ptr %q, i64 4, i1 false)"),
        ],
        span: Span::dummy(),
    });
    assert_eq!(
        raw_body_memory_operations(&llvm),
        vec![
            RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Alloca),
            RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Load),
            RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Store),
            RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Fence),
            RawBodyMemoryOp::Llvm(LlvmRawBodyMemoryOp::Memcpy),
        ]
    );
}

#[test]
fn raw_body_direct_callee_effects_are_typed_before_consumers() {
    let wasm = HirBody::Wasm(WasmBlock {
        lines: vec![
            String::from("call $load_i32"),
            String::from("call $fd_write"),
            String::from("call $custom_helper"),
        ],
        span: Span::dummy(),
    });
    assert_eq!(
        raw_body_direct_callee_effects(&wasm),
        vec![
            RawBodyDirectCallee::RawMemory {
                callee: String::from("load_i32"),
                operation: RawMemoryOp::Load,
            },
            RawBodyDirectCallee::Other(String::from("fd_write")),
            RawBodyDirectCallee::Other(String::from("custom_helper")),
        ]
    );

    let llvm = HirBody::LlvmIr(LlvmIrBlock {
        lines: vec![
            String::from("call void @store_i32(i32 0, i32 1)"),
            String::from("call void @llvm.assume(i1 true)"),
            String::from("call float @llvm.sqrt.f32(float 4.0)"),
            String::from("call void @llvm.trap()"),
            String::from("call i32 @fd_read(i32 0)"),
        ],
        span: Span::dummy(),
    });
    assert_eq!(
        raw_body_direct_callee_effects(&llvm),
        vec![
            RawBodyDirectCallee::RawMemory {
                callee: String::from("store_i32"),
                operation: RawMemoryOp::Store,
            },
            RawBodyDirectCallee::BackendIntrinsic {
                callee: String::from("llvm.assume"),
                intrinsic: RawBodyBackendIntrinsic::Llvm(LlvmRawBodyIntrinsic::Assume),
            },
            RawBodyDirectCallee::BackendIntrinsic {
                callee: String::from("llvm.sqrt.f32"),
                intrinsic: RawBodyBackendIntrinsic::Llvm(LlvmRawBodyIntrinsic::Sqrt),
            },
            RawBodyDirectCallee::Other(String::from("llvm.trap")),
            RawBodyDirectCallee::Other(String::from("fd_read")),
        ]
    );
}

#[test]
fn pure_wasm_raw_comment_with_impure_marker_is_allowed() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_const <()->i32> ():
    #wasm:
        ;; fd_write is only documentation here
        i32.const 7

fn main <()->i32> ():
    raw_const
"#;

    compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).expect("compile");
}

#[test]
fn pure_wasm_raw_direct_impure_marker_call_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_io <()->i32> ():
    #wasm:
        i32.const 0
        call $fd_write
        drop
        i32.const 0

fn main <()->i32> ():
    raw_io
"#;

    let result = compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).map(|_| ());
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_wasm_raw_memory_store_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store

fn main <()->i32> ():
    raw_store 0 1
    0
"#;

    let result = compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).map(|_| ());
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_wasm_raw_memory_grow_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_grow <(i32)->i32> (pages):
    #wasm:
        local.get pages
        memory.grow

fn main <()->i32> ():
    raw_grow 1
"#;

    let result = compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).map(|_| ());
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_wasm_raw_call_to_raw_memory_helper_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

#extern "env" "store_i32" fn store_i32 <(i32,i32)->()>

fn raw_store_helper <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        call $store_i32

fn main <()->i32> ():
    raw_store_helper 0 1
    0
"#;

    let result = compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).map(|_| ());
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_wasm_raw_call_to_suffixed_raw_memory_helper_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

#extern "env" "store_i32__i32_i32__unit__pure" fn raw_store_symbol <(i32,i32)->()>

fn raw_store_helper <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        call $store_i32__i32_i32__unit__pure

fn main <()->i32> ():
    raw_store_helper 0 1
    0
"#;

    let result = compile_wasm(FileId(0), src, options(CompileTarget::Wasm)).map(|_| ());
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_raw_load_intrinsic_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_load <()->i32> ():
    #intrinsic "load" <i32> (16)

fn main <()->i32> ():
    raw_load
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Wasm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_indirect_impure_function_value_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn impure_id <(i32)*>i32> (x):
    x

fn call_callback <((i32)*>i32, i32)->i32> (callback, value):
    callback value

fn main <()->i32> ():
    call_callback @impure_id 1
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Wasm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_raw_store_intrinsic_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn raw_store <()->i32> ():
    #intrinsic "store" <i32> (16, 1)
    0

fn main <()->i32> ():
    raw_store
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Wasm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn raw_memory_intrinsic_in_core_mem_source_is_allowed_during_migration() {
    let src = r#"
#entry load_i32
#no_prelude
#indent 4
#target wasm

trait Copy:
    #capability clone
    #capability copy
    fn clone <(Self)->Self> (x):
        x

    fn copy_mark <(Self)->Self> (x):
        x

impl Copy for i32:
    fn clone <(i32)->i32> (x):
        x

    fn copy_mark <(i32)->i32> (x):
        x

fn load_i32 <(i32)->i32> (p):
    #intrinsic "load" <i32> (p)
"#;

    check_source_as_core_mem_boundary(src, "C:/repo/stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("core/mem intrinsic helper remains allowed during migration");
}

#[test]
fn collection_slot_lifecycle_intrinsic_requires_source_evidence() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

pub struct MemPtr<.T>:
    raw <i32>

fn helper <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <i32> (ptr, offset)
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Wasm),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted,
        ),
    );
}

#[test]
fn collection_slot_lifecycle_intrinsic_accepts_matching_stdlib_anchor() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

trait Copy:
    #capability clone
    #capability copy
    fn clone <(Self)->Self> (x):
        x

    fn copy_mark <(Self)->Self> (x):
        x

impl Copy for i32:
    fn clone <(i32)->i32> (x):
        x

    fn copy_mark <(i32)->i32> (x):
        x

fn helper <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <i32> (ptr, offset)
"#;

    check_source_with_canonical_mem_types(
        src,
        "alloc/collections/vec/slot_boundary.nepl",
        CompileTarget::Wasm,
    )
    .expect("stdlib collection slot intrinsic with matching anchor type is allowed");
}

#[test]
fn collection_slot_lifecycle_intrinsic_rejects_public_stdlib_callable_surface() {
    let src = r#"
#entry public_slot
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

pub fn public_slot <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <i32> (ptr, offset)
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted,
        ),
    );
}

#[test]
fn collection_slot_lifecycle_intrinsic_rejects_public_alias_surface() {
    let src = r#"
#entry internal_slot
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn internal_slot <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <i32> (ptr, offset)

pub fn public_slot internal_slot;
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted,
        ),
    );
}

#[test]
fn collection_slot_lifecycle_intrinsic_rejects_public_wrapper_reachability() {
    let src = r#"
#entry public_slot
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn internal_slot <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <i32> (ptr, offset)

pub fn public_slot <(MemPtr<i32>,i32)->()> (ptr, offset):
    internal_slot ptr offset
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted,
        ),
    );
}

#[test]
fn collection_slot_lifecycle_intrinsic_rejects_anchor_type_mismatch() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(MemPtr<i32>,i32)->()> (ptr, offset):
    #intrinsic "collection_slot_initialize_empty" <u8> (ptr, offset)
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::IntrinsicArgTypeMismatch,
        ),
    );
}

#[test]
fn collection_slot_storage_dealloc_requires_owner_token_anchor() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(MemPtr<i32>)->()> (ptr):
    #intrinsic "collection_slot_storage_dealloc" <> (ptr)
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::IntrinsicArgTypeMismatch,
        ),
    );
}

#[test]
fn collection_slot_storage_relocate_accepts_matching_owner_tokens() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(RegionToken<i32>,RegionToken<i32>)->()> (old, new):
    #intrinsic "collection_slot_storage_relocate" <> (old, new)
"#;

    check_source_with_canonical_mem_types(
        src,
        "alloc/collections/vec/slot_boundary.nepl",
        CompileTarget::Wasm,
    )
    .expect("storage relocate with matching owner token element types is allowed");
}

#[test]
fn collection_slot_storage_relocate_rejects_mismatched_owner_tokens() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(RegionToken<i32>,RegionToken<u8>)->()> (old, new):
    #intrinsic "collection_slot_storage_relocate" <> (old, new)
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::IntrinsicArgTypeMismatch,
        ),
    );
}

#[test]
fn collection_slot_drop_traversal_accepts_matching_owner_token_type() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(RegionToken<i32>)->()> (storage):
    #intrinsic "collection_slot_drop_traversal" <i32> (storage)
"#;

    check_source_with_canonical_mem_types(
        src,
        "alloc/collections/vec/slot_boundary.nepl",
        CompileTarget::Wasm,
    )
    .expect("drop traversal with matching owner token element type is allowed");
}

#[test]
fn collection_slot_drop_traversal_rejects_mismatched_owner_token_type() {
    let src = r#"
#entry helper
#no_prelude
#indent 4
#target wasm

#import "core/mem/types" as *

fn helper <(RegionToken<i32>)->()> (storage):
    #intrinsic "collection_slot_drop_traversal" <u8> (storage)
"#;

    assert_has_diag(
        check_source_with_canonical_mem_types(
            src,
            "alloc/collections/vec/slot_boundary.nepl",
            CompileTarget::Wasm,
        ),
        DiagnosticCode::Type(
            nepl_core::diagnostic_codes::TypeDiagnosticCode::IntrinsicArgTypeMismatch,
        ),
    );
}

#[test]
fn pure_llvm_raw_comment_with_impure_marker_is_allowed() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn raw_const <()->i32> ():
    #llvmir:
        define i32 @raw_const() {
        entry:
            ; fd_write is only documentation here
            ret i32 7
        }

fn main <()->i32> ():
    raw_const
"#;

    check_source(src, CompileTarget::Llvm).expect("check");
}

#[test]
fn pure_llvm_raw_call_to_declared_pure_substring_name_is_allowed() {
    let src = r#"
#entry main
#indent 4
#target llvm

#extern "c" "fd_write_like" fn fd_write_like <()->i32>

fn raw_call <()->i32> ():
    #llvmir:
        define i32 @raw_call() {
        entry:
            %x = call i32 @fd_write_like()
            ret i32 %x
        }

fn main <()->i32> ():
    raw_call
"#;

    check_source(src, CompileTarget::Llvm).expect("check");
}

#[test]
fn pure_llvm_raw_call_to_known_pure_backend_intrinsic_is_allowed() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn raw_sqrt <(f32)->f32> (x):
    #llvmir:
        define float @raw_sqrt(float %x) {
        entry:
            %y = call float @llvm.sqrt.f32(float %x)
            ret float %y
        }

fn main <()->f32> ():
    raw_sqrt 4.0
"#;

    check_source(src, CompileTarget::Llvm).expect("check");
}

#[test]
fn pure_llvm_raw_call_to_unknown_llvm_intrinsic_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn raw_unknown <()->i32> ():
    #llvmir:
        define i32 @raw_unknown() {
        entry:
            call void @llvm.trap()
            ret i32 0
        }

fn main <()->i32> ():
    raw_unknown
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Llvm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_llvm_raw_call_to_declared_impure_extern_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target llvm

#extern "c" "fd_write" fn fd_write <()*>i32>

fn raw_io <()->i32> ():
    #llvmir:
        define i32 @raw_io() {
        entry:
            %x = call i32 @fd_write()
            ret i32 %x
        }

fn main <()->i32> ():
    raw_io
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Llvm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_llvm_raw_memory_store_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn raw_store <(i32)->()> (v):
    #llvmir:
        define void @raw_store(i32 %v) {
        entry:
            %p = alloca i32
            store i32 %v, ptr %p, align 4
            ret void
        }

fn main <()->i32> ():
    raw_store 1
    0
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Llvm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_llvm_raw_call_to_raw_memory_helper_is_rejected_outside_core_mem() {
    let src = r#"
#entry main
#indent 4
#target llvm

#extern "c" "mem_grow" fn mem_grow <(i32)->i32>

fn raw_grow_helper <(i32)->i32> (pages):
    #llvmir:
        define i32 @raw_grow_helper(i32 %pages) {
        entry:
            %x = call i32 @mem_grow(i32 %pages)
            ret i32 %x
        }

fn main <()->i32> ():
    raw_grow_helper 1
"#;

    assert_has_diag(
        check_source(src, CompileTarget::Llvm),
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_raw_memory_in_core_mem_source_is_allowed_during_migration() {
    let src = r#"
#entry raw_store
#no_prelude
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#;

    check_source_as_core_mem_boundary(src, "C:/repo/stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("core/mem raw memory helper remains allowed during migration");
}

#[test]
fn pure_raw_body_call_to_raw_helper_in_core_mem_source_is_allowed_during_migration() {
    let src = r#"
#entry raw_store_helper
#no_prelude
#indent 4
#target wasm

fn store_i32 <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store

fn raw_store_helper <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        call $store_i32
"#;

    check_source_as_core_mem_boundary(src, "C:/repo/stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("core/mem raw memory helper call remains allowed during migration");
}

#[test]
fn pure_raw_memory_path_suffix_without_capability_is_rejected() {
    let src = r#"
#entry raw_store
#no_prelude
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#;

    let result =
        check_source_with_path(src, "/tmp/custom_stdlib/core/mem.nepl", CompileTarget::Wasm);
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}

#[test]
fn pure_raw_memory_in_custom_stdlib_core_mem_source_is_allowed() {
    let src = r#"
#entry raw_store
#no_prelude
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#;

    check_source_as_core_mem_boundary(src, "/tmp/custom_stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("custom stdlib roots still provide the core/mem raw memory boundary");
}

#[test]
fn loader_does_not_mark_configured_stdlib_core_mem_facade_as_raw_memory_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    std::fs::create_dir_all(stdlib_root.join("core")).expect("create stdlib core dir");
    std::fs::write(
        stdlib_root.join("core").join("mem.nepl"),
        r#"
#indent 4
#target wasm

pub fn safe_value <()->i32> ():
    7
"#,
    )
    .expect("write core mem");
    let entry = temp.path().join("main.nepl");
    std::fs::write(
        &entry,
        r#"
#entry main
#indent 4
#target wasm
#no_prelude

#import "core/mem" as *

fn main <()->i32> ():
    safe_value
"#,
    )
    .expect("write entry");

    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load(&entry).expect("load");
    let file_id = source_file_id_for_suffix(&loaded.source_map, "stdlib/core/mem.nepl");
    let probe = source_capability_probe_span(file_id);
    assert!(
        !loaded
            .source_map
            .raw_memory_structural_boundary_allowed_at(probe)
            && !loaded
                .source_map
                .raw_memory_operation_boundary_allowed_at(probe, RawMemoryOp::Store),
        "core/mem facade without raw source evidence must not receive raw memory capability"
    );
    check_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        options(CompileTarget::Wasm),
    )
    .expect("safe facade source should check");
}

#[test]
fn loader_does_not_mark_configured_stdlib_alloc_string_facade_as_raw_memory_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    std::fs::create_dir_all(stdlib_root.join("alloc")).expect("create stdlib alloc dir");
    std::fs::write(
        stdlib_root.join("alloc").join("string.nepl"),
        r#"
#indent 4
#target wasm

pub fn string_safe_value <()->i32> ():
    11
"#,
    )
    .expect("write alloc string");
    let entry = temp.path().join("main.nepl");
    std::fs::write(
        &entry,
        r#"
#entry main
#indent 4
#target wasm
#no_prelude

#import "alloc/string" as *

fn main <()->i32> ():
    string_safe_value
"#,
    )
    .expect("write entry");

    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load(&entry).expect("load");
    let file_id = source_file_id_for_suffix(&loaded.source_map, "stdlib/alloc/string.nepl");
    let probe = source_capability_probe_span(file_id);
    assert!(
        !loaded
            .source_map
            .raw_memory_structural_boundary_allowed_at(probe)
            && !loaded
                .source_map
                .raw_memory_operation_boundary_allowed_at(probe, RawMemoryOp::Store),
        "alloc/string facade without raw source evidence must not receive raw memory capability"
    );
    check_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        options(CompileTarget::Wasm),
    )
    .expect("safe facade source should check");
}

#[test]
fn loader_marks_configured_stdlib_alloc_string_storage_as_raw_memory_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    let storage_dir = stdlib_root.join("alloc").join("string");
    std::fs::create_dir_all(&storage_dir).expect("create stdlib alloc/string dir");
    std::fs::write(
        storage_dir.join("storage.nepl"),
        r#"
#indent 4
#target wasm

pub fn string_storage_raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#,
    )
    .expect("write alloc string storage");
    let entry = temp.path().join("main.nepl");
    std::fs::write(
        &entry,
        r#"
#entry main
#indent 4
#target wasm
#no_prelude

#import "alloc/string/storage" as *

fn main <()->i32> ():
    string_storage_raw_store 0 1;
    0
"#,
    )
    .expect("write entry");

    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load(&entry).expect("load");
    check_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        options(CompileTarget::Wasm),
    )
    .expect("configured stdlib alloc/string/storage has raw memory capability");
}

#[test]
fn loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary() {
    let cases: &[(&[&str], &str, &str)] = &[
        (
            &["alloc", "io", "bytebuf.nepl"],
            "alloc/io/bytebuf",
            "alloc_io_bytebuf_raw_store",
        ),
        (
            &["alloc", "io", "bytebuilder", "storage.nepl"],
            "alloc/io/bytebuilder/storage",
            "alloc_io_bytebuilder_storage_raw_store",
        ),
        (
            &["alloc", "io", "bytebuilder", "append.nepl"],
            "alloc/io/bytebuilder/append",
            "alloc_io_bytebuilder_append_raw_store",
        ),
        (
            &["alloc", "io", "bytebuilder", "build.nepl"],
            "alloc/io/bytebuilder/build",
            "alloc_io_bytebuilder_build_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "mutation", "push.nepl"],
            "alloc/collections/vec/mutation/push",
            "alloc_collections_vec_mutation_push_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "raw", "element.nepl"],
            "alloc/collections/vec/raw/element",
            "alloc_collections_vec_raw_element_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "storage", "alloc.nepl"],
            "alloc/collections/vec/storage/alloc",
            "alloc_collections_vec_storage_alloc_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "storage", "cleanup.nepl"],
            "alloc/collections/vec/storage/cleanup",
            "alloc_collections_vec_storage_cleanup_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "storage", "fill.nepl"],
            "alloc/collections/vec/storage/fill",
            "alloc_collections_vec_storage_fill_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "sort", "common.nepl"],
            "alloc/collections/vec/sort/common",
            "alloc_collections_vec_sort_common_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "sort", "merge", "api.nepl"],
            "alloc/collections/vec/sort/merge/api",
            "alloc_collections_vec_sort_merge_api_raw_store",
        ),
        (
            &["alloc", "string", "access.nepl"],
            "alloc/string/access",
            "alloc_string_access_raw_store",
        ),
        (
            &["alloc", "string", "builder", "append.nepl"],
            "alloc/string/builder/append",
            "alloc_string_builder_append_raw_store",
        ),
        (
            &["alloc", "string", "builder", "build.nepl"],
            "alloc/string/builder/build",
            "alloc_string_builder_build_raw_store",
        ),
        (
            &["alloc", "string", "builder", "reserve.nepl"],
            "alloc/string/builder/reserve",
            "alloc_string_builder_reserve_raw_store",
        ),
        (
            &["alloc", "string", "builder", "types.nepl"],
            "alloc/string/builder/types",
            "alloc_string_builder_types_raw_store",
        ),
        (
            &["alloc", "string", "builder_ext.nepl"],
            "alloc/string/builder_ext",
            "alloc_string_builder_ext_raw_store",
        ),
        (
            &["alloc", "string", "concat.nepl"],
            "alloc/string/concat",
            "alloc_string_concat_raw_store",
        ),
        (
            &["alloc", "string", "scanner.nepl"],
            "alloc/string/scanner",
            "alloc_string_scanner_raw_store",
        ),
        (
            &["alloc", "string", "utf8.nepl"],
            "alloc/string/utf8",
            "alloc_string_utf8_raw_store",
        ),
        (
            &["std", "text", "validate.nepl"],
            "std/text/validate",
            "std_text_validate_raw_store",
        ),
        (
            &["std", "streamio", "scanner", "state.nepl"],
            "std/streamio/scanner/state",
            "std_streamio_scanner_state_raw_store",
        ),
    ];

    for (segments, import_spec, function_name) in cases {
        let temp = tempfile::tempdir().expect("tempdir");
        let stdlib_root = temp.path().join("stdlib");
        let source_path = segments
            .iter()
            .fold(stdlib_root.clone(), |path, segment| path.join(segment));
        std::fs::create_dir_all(source_path.parent().expect("source parent"))
            .expect("create stdlib boundary dir");
        std::fs::write(
            &source_path,
            format!(
                r#"
#indent 4
#target wasm

pub fn {function_name} <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#
            ),
        )
        .expect("write stdlib boundary module");
        let entry = temp.path().join("main.nepl");
        std::fs::write(
            &entry,
            format!(
                r#"
#entry main
#indent 4
#target wasm
#no_prelude

#import "{import_spec}" as *

fn main <()->i32> ():
    {function_name} 0 1;
    0
"#
            ),
        )
        .expect("write entry");

        let mut loader = Loader::new(stdlib_root);
        let loaded = loader.load(&entry).expect("load");
        check_module_with_source_map(
            loaded.module,
            Some(&loaded.source_map),
            options(CompileTarget::Wasm),
        )
        .unwrap_or_else(|err| panic!("{import_spec} should have raw memory capability: {err:?}"));
    }
}

#[test]
fn loader_does_not_mark_raw_memory_free_split_modules_as_raw_memory_boundaries() {
    let cases: &[(&[&str], &str, &str)] = &[
        (
            &["alloc", "collections", "vec.nepl"],
            "alloc/collections/vec",
            "alloc_collections_vec_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "access.nepl"],
            "alloc/collections/vec/access",
            "alloc_collections_vec_access_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "access", "data.nepl"],
            "alloc/collections/vec/access/data",
            "alloc_collections_vec_access_data_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "access", "header.nepl"],
            "alloc/collections/vec/access/header",
            "alloc_collections_vec_access_header_raw_store",
        ),
        (&["std", "text.nepl"], "std/text", "std_text_raw_store"),
        (
            &["alloc", "collections", "vec", "mutation.nepl"],
            "alloc/collections/vec/mutation",
            "alloc_collections_vec_mutation_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "mutation", "cleanup.nepl"],
            "alloc/collections/vec/mutation/cleanup",
            "alloc_collections_vec_mutation_cleanup_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "mutation", "pop.nepl"],
            "alloc/collections/vec/mutation/pop",
            "alloc_collections_vec_mutation_pop_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "mutation", "replace.nepl"],
            "alloc/collections/vec/mutation/replace",
            "alloc_collections_vec_mutation_replace_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "storage.nepl"],
            "alloc/collections/vec/storage",
            "alloc_collections_vec_storage_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "storage", "view.nepl"],
            "alloc/collections/vec/storage/view",
            "alloc_collections_vec_storage_view_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "raw.nepl"],
            "alloc/collections/vec/raw",
            "alloc_collections_vec_raw_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "raw", "aggregate.nepl"],
            "alloc/collections/vec/raw/aggregate",
            "alloc_collections_vec_raw_aggregate_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "raw", "predicate.nepl"],
            "alloc/collections/vec/raw/predicate",
            "alloc_collections_vec_raw_predicate_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "raw", "prefix.nepl"],
            "alloc/collections/vec/raw/prefix",
            "alloc_collections_vec_raw_prefix_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "transform.nepl"],
            "alloc/collections/vec/transform",
            "alloc_collections_vec_transform_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "transform", "filter.nepl"],
            "alloc/collections/vec/transform/filter",
            "alloc_collections_vec_transform_filter_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "transform", "map.nepl"],
            "alloc/collections/vec/transform/map",
            "alloc_collections_vec_transform_map_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "transform", "prefix.nepl"],
            "alloc/collections/vec/transform/prefix",
            "alloc_collections_vec_transform_prefix_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "types.nepl"],
            "alloc/collections/vec/types",
            "alloc_collections_vec_types_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "query.nepl"],
            "alloc/collections/vec/query",
            "alloc_collections_vec_query_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "query", "aggregate.nepl"],
            "alloc/collections/vec/query/aggregate",
            "alloc_collections_vec_query_aggregate_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "query", "get.nepl"],
            "alloc/collections/vec/query/get",
            "alloc_collections_vec_query_get_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "query", "predicate.nepl"],
            "alloc/collections/vec/query/predicate",
            "alloc_collections_vec_query_predicate_raw_store",
        ),
        (
            &["alloc", "string", "builder.nepl"],
            "alloc/string/builder",
            "alloc_string_builder_raw_store",
        ),
        (
            &["alloc", "io", "bytebuilder.nepl"],
            "alloc/io/bytebuilder",
            "alloc_io_bytebuilder_raw_store",
        ),
        (
            &["alloc", "io", "bytebuilder", "types.nepl"],
            "alloc/io/bytebuilder/types",
            "alloc_io_bytebuilder_types_raw_store",
        ),
        (
            &["core", "mem", "types.nepl"],
            "core/mem/types",
            "core_mem_types_raw_store",
        ),
        (
            &["alloc", "string", "float.nepl"],
            "alloc/string/float",
            "alloc_string_float_raw_store",
        ),
        (
            &["alloc", "string", "integer.nepl"],
            "alloc/string/integer",
            "alloc_string_integer_raw_store",
        ),
    ];

    for (segments, import_spec, function_name) in cases {
        let temp = tempfile::tempdir().expect("tempdir");
        let stdlib_root = temp.path().join("stdlib");
        let source_path = segments
            .iter()
            .fold(stdlib_root.clone(), |path, segment| path.join(segment));
        std::fs::create_dir_all(source_path.parent().expect("source parent"))
            .expect("create stdlib facade dir");
        std::fs::write(
            &source_path,
            format!(
                r#"
#indent 4
#target wasm

pub fn {function_name} <()->i32> ():
    1
"#
            ),
        )
        .expect("write stdlib facade module");
        let entry = temp.path().join("main.nepl");
        std::fs::write(
            &entry,
            format!(
                r#"
#entry main
#indent 4
#target wasm
#no_prelude

#import "{import_spec}" as *

fn main <()->i32> ():
    {function_name}
"#
            ),
        )
        .expect("write entry");

        let mut loader = Loader::new(stdlib_root);
        let loaded = loader.load(&entry).expect("load");
        let expected_suffix = format!("stdlib/{import_spec}.nepl");
        let file_id = source_file_id_for_suffix(&loaded.source_map, expected_suffix.as_str());
        let probe = source_capability_probe_span(file_id);
        assert!(
            !loaded
                .source_map
                .raw_memory_structural_boundary_allowed_at(probe)
                && !loaded
                    .source_map
                    .raw_memory_operation_boundary_allowed_at(probe, RawMemoryOp::Store),
            "{import_spec} without raw source evidence must not receive raw memory capability"
        );
        check_module_with_source_map(
            loaded.module,
            Some(&loaded.source_map),
            options(CompileTarget::Wasm),
        )
        .unwrap_or_else(|err| panic!("{import_spec} safe source should check: {err:?}"));
    }
}

#[test]
fn loader_does_not_mark_user_core_mem_path_by_suffix() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    std::fs::create_dir_all(&stdlib_root).expect("create stdlib root");
    let user_core_dir = temp.path().join("user").join("core");
    std::fs::create_dir_all(&user_core_dir).expect("create user core dir");
    let entry = user_core_dir.join("mem.nepl");
    std::fs::write(
        &entry,
        r#"
#entry raw_store
#indent 4
#target wasm
#no_prelude

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
"#,
    )
    .expect("write user core mem");

    let mut loader = Loader::new(stdlib_root);
    let loaded = loader.load(&entry).expect("load");
    let result = check_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        options(CompileTarget::Wasm),
    );
    assert_has_diag(
        result,
        DiagnosticCode::Effect(nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure),
    );
}
