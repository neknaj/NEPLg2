use nepl_core::diagnostic_codes::{DiagnosticCode, LexerDiagnosticCode};
use nepl_core::lexer;
use nepl_core::span::FileId;

#[test]
fn lexer_reports_dedent_to_unknown_indent_without_panic() {
    let source = concat!(
        "#indent 4\n",
        "fn main %fn void i32 \\void:\n",
        "    if true:\n",
        "        then:\n",
        "            1\n",
        "       else:\n",
        "            0\n",
    );

    let lexed = lexer::lex(FileId(0), source);

    assert!(
        lexed.diagnostics.iter().any(|diagnostic| diagnostic.code
            == DiagnosticCode::Lexer(LexerDiagnosticCode::IndentLevelMismatch)),
        "expected indent mismatch diagnostic, got {:?}",
        lexed.diagnostics,
    );
}
