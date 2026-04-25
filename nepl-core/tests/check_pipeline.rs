use nepl_core::error::CoreError;
use nepl_core::span::FileId;
use nepl_core::{check_module, lexer, parser, CompileOptions};

fn parse_module(source: &str) -> nepl_core::ast::Module {
    let lexed = lexer::lex(FileId(0), source);
    let parsed = parser::parse_tokens(FileId(0), lexed);
    assert!(
        parsed.diagnostics.is_empty(),
        "parser diagnostics: {:?}",
        parsed.diagnostics
    );
    parsed.module.expect("module should parse")
}

fn deep_identity_source(call_count: usize) -> String {
    let mut source = String::from(
        "#entry main\n#indent 4\n#target core\n\nfn inc <(i32)->i32> (x):\n    x\n\nfn main <()->i32> ():\n    ",
    );
    for _ in 0..call_count {
        source.push_str("inc ");
    }
    source.push_str("0\n");
    source
}

#[test]
fn check_module_accepts_deep_prefix_chain_without_codegen_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));

    check_module(module, CompileOptions::default())
        .expect("check-only pipeline should not enter recursive artifact generation");
}

#[test]
fn check_module_reports_type_errors() {
    let module = parse_module(
        "#entry main\n#indent 4\n#target core\n\nfn main <()->i32> ():\n    unknown_symbol\n",
    );

    let err = check_module(module, CompileOptions::default())
        .expect_err("typecheck diagnostics should fail check-only pipeline");
    assert!(matches!(err, CoreError::Diagnostics(_)));
}
