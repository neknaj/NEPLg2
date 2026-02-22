use nepl_core::ast::{FnBody, Stmt};
use nepl_core::diagnostic::Severity;
use nepl_core::lexer;
use nepl_core::parser;
use nepl_core::span::FileId;

#[test]
fn llvmir_block_allows_internal_indentation_as_raw_text() {
    let src = r#"
#indent 4
#target llvm

#llvmir:
    ; module level
      ; deeper comment
    define i32 @m() {
    entry:
      ret i32 0
    }
"#;

    let lex = lexer::lex(FileId(0), src);
    assert!(
        lex.diagnostics
            .iter()
            .all(|d| !matches!(d.severity, Severity::Error)),
        "unexpected lexer errors: {:?}",
        lex.diagnostics
    );

    let parse = parser::parse_tokens(FileId(0), lex);
    assert!(
        parse.diagnostics
            .iter()
            .all(|d| !matches!(d.severity, Severity::Error)),
        "unexpected parser errors: {:?}",
        parse.diagnostics
    );

    let module = parse.module.expect("module should parse");
    let stmt = module
        .root
        .items
        .iter()
        .find(|s| matches!(s, Stmt::LlvmIr(_)))
        .expect("top-level llvmir stmt should exist");
    let Stmt::LlvmIr(block) = stmt else {
        panic!("expected llvmir stmt");
    };

    assert!(
        block.lines.iter().any(|l| l.starts_with("  ; deeper comment")),
        "internal indent must be preserved in raw llvmir text: {:?}",
        block.lines
    );
    assert!(
        block.lines.iter().any(|l| l.starts_with("  ret i32 0")),
        "ret indentation must be preserved: {:?}",
        block.lines
    );
}

#[test]
fn llvmir_function_body_is_recognized() {
    let src = r#"
#indent 4
#target llvm

fn f <()->i32> ():
    #llvmir:
        define i32 @f() {
        entry:
          ret i32 7
        }
"#;

    let lex = lexer::lex(FileId(0), src);
    let parse = parser::parse_tokens(FileId(0), lex);
    assert!(
        parse.diagnostics
            .iter()
            .all(|d| !matches!(d.severity, Severity::Error)),
        "unexpected parser errors: {:?}",
        parse.diagnostics
    );

    let module = parse.module.expect("module should parse");
    let fdef = module
        .root
        .items
        .iter()
        .find_map(|s| match s {
            Stmt::FnDef(def) if def.name.name == "f" => Some(def),
            _ => None,
        })
        .expect("fn f should exist");

    match &fdef.body {
        FnBody::LlvmIr(block) => {
            assert!(
                block.lines.iter().any(|l| l.contains("define i32 @f()")),
                "llvmir function body should contain define line: {:?}",
                block.lines
            );
        }
        other => panic!("expected FnBody::LlvmIr, got {:?}", other),
    }
}
