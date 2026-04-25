use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_ids::DiagnosticId;
use nepl_core::error::CoreError;
use nepl_core::span::FileId;
use nepl_core::{check_module, compile_wasm, lexer, parser, CompileOptions, CompileTarget};

fn options(target: CompileTarget) -> CompileOptions {
    CompileOptions {
        target: Some(target),
        verbose: false,
        profile: None,
    }
}

fn parse_module(src: &str) -> nepl_core::ast::Module {
    let lex = lexer::lex(FileId(0), src);
    let parse = parser::parse_tokens(FileId(0), lex);
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
