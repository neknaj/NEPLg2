use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_ids::DiagnosticId;
use nepl_core::error::CoreError;
use nepl_core::source_map::SourceMap;
use nepl_core::span::FileId;
use nepl_core::{
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

fn assert_has_diag(result: Result<(), CoreError>, id: DiagnosticId) {
    match result {
        Err(CoreError::Diagnostics(diags)) => assert!(
            diags.iter().any(|d| d.id == Some(id)),
            "expected diagnostic {:?}, got {:?}",
            id,
            diags
        ),
        other => panic!("expected diagnostics, got {:?}", other),
    }
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
    assert_has_diag(result, DiagnosticId::TypePureCallsImpureFunction);
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
    assert_has_diag(result, DiagnosticId::TypePureCallsImpureFunction);
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
    assert_has_diag(result, DiagnosticId::TypePureCallsImpureFunction);
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
        DiagnosticId::TypePureCallsImpureFunction,
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
        DiagnosticId::TypePureCallsImpureFunction,
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

    check_source_with_path(src, "C:/repo/stdlib/core/mem.nepl", CompileTarget::Wasm)
        .expect("core/mem raw memory helper remains allowed during migration");
}
