use nepl_core::ast::Effect;
use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::DiagnosticCode;
use nepl_core::effects::{
    external_io_op_from_name, internal_effect_surface_fold, intrinsic_effect, nondet_op_from_name,
    raw_body_memory_operations, raw_callee_internal_effect, raw_memory_callee_internal_effect,
    raw_memory_op_from_name, ExternalIoOp, InternalEffect, LlvmRawBodyMemoryOp, NondetOp,
    RawBodyMemoryOp, RawMemoryOp, WasmRawBodyMemoryOp,
};
use nepl_core::error::CoreError;
use nepl_core::hir::HirBody;
use nepl_core::loader::Loader;
use nepl_core::source_map::{SourceCapabilities, SourceMap};
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
    let mut source_map = SourceMap::new();
    let file_id = source_map.add_with_capabilities(
        path,
        String::from(src),
        SourceCapabilities::raw_memory_boundary(),
    );
    let module = parse_module_with_file_id(file_id, src);
    check_module_with_source_map(module, Some(&source_map), options(target))
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
fn all_impure_io_effect_markers_have_typed_operations() {
    for marker in nepl_core::effects::IMPURE_IO_EFFECT_MARKERS {
        assert!(
            external_io_op_from_name(marker).is_some() || nondet_op_from_name(marker).is_some(),
            "impure host marker '{}' must map to ExternalIoOp or NondetOp",
            marker
        );
    }
}

#[test]
fn all_raw_memory_effect_markers_have_typed_operations() {
    for marker in nepl_core::effects::RAW_MEMORY_HELPER_EFFECT_MARKERS
        .iter()
        .chain(nepl_core::effects::RAW_MEMORY_INTRINSIC_EFFECT_MARKERS.iter())
    {
        assert!(
            raw_memory_op_from_name(marker).is_some(),
            "raw memory marker '{}' must map to RawMemoryOp",
            marker
        );
    }
}

#[test]
fn raw_body_memory_operations_are_typed_by_backend() {
    let wasm = HirBody::Wasm(WasmBlock {
        lines: vec![
            String::from("i32.load"),
            String::from("i64.store"),
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
#indent 4
#target wasm

fn load_i32 <(i32)->i32> (p):
    #intrinsic "load" <i32> (p)
"#;

    check_source_as_core_mem_boundary(src, "C:/repo/stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("core/mem intrinsic helper remains allowed during migration");
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
fn loader_marks_configured_stdlib_core_mem_as_raw_memory_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    std::fs::create_dir_all(stdlib_root.join("core")).expect("create stdlib core dir");
    std::fs::write(
        stdlib_root.join("core").join("mem.nepl"),
        r#"
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
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
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    raw_store 0 1;
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
    .expect("configured stdlib core/mem has raw memory capability");
}

#[test]
fn loader_marks_configured_stdlib_alloc_string_as_raw_memory_boundary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let stdlib_root = temp.path().join("stdlib");
    std::fs::create_dir_all(stdlib_root.join("alloc")).expect("create stdlib alloc dir");
    std::fs::write(
        stdlib_root.join("alloc").join("string.nepl"),
        r#"
#indent 4
#target wasm

fn string_raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
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
    string_raw_store 0 1;
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
    .expect("configured stdlib alloc/string has raw memory capability");
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

fn string_storage_raw_store <(i32,i32)->()> (p, v):
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
            &["alloc", "io", "bytebuilder.nepl"],
            "alloc/io/bytebuilder",
            "alloc_io_bytebuilder_raw_store",
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
            &[
                "alloc",
                "collections",
                "vec",
                "sort",
                "merge",
                "buffer.nepl",
            ],
            "alloc/collections/vec/sort/merge/buffer",
            "alloc_collections_vec_sort_merge_buffer_raw_store",
        ),
        (
            &["alloc", "collections", "vec", "sort", "merge", "range.nepl"],
            "alloc/collections/vec/sort/merge/range",
            "alloc_collections_vec_sort_merge_range_raw_store",
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
            &["alloc", "string", "float.nepl"],
            "alloc/string/float",
            "alloc_string_float_raw_store",
        ),
        (
            &["alloc", "string", "integer.nepl"],
            "alloc/string/integer",
            "alloc_string_integer_raw_store",
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

fn {function_name} <(i32,i32)->()> (p, v):
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

fn {function_name} <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store
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
    {function_name} 0 1;
    0
"#
            ),
        )
        .expect("write entry");

        let mut loader = Loader::new(stdlib_root);
        let loaded = loader.load(&entry).expect("load");
        let result = check_module_with_source_map(
            loaded.module,
            Some(&loaded.source_map),
            options(CompileTarget::Wasm),
        );
        assert_has_diag(
            result,
            DiagnosticCode::Effect(
                nepl_core::diagnostic_codes::EffectDiagnosticCode::PureCallsImpure,
            ),
        );
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
