mod harness;

use harness::run_main_i32;
use nepl_core::diagnostic::Severity;
use nepl_core::lexer::{self, TokenKind};
use nepl_core::loader::Loader;
use nepl_core::span::FileId;
use nepl_core::{compile_module_with_source_map, BuildProfile, CompileOptions, CompileTarget};

fn stdlib_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

fn compile_with_loader(src: &str) -> Result<(), nepl_core::error::CoreError> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("char_test.nepl".into(), src.to_string())
        .expect("load");
    compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    )
    .map(|_| ())
}

fn emit_llvm_with_loader(src: &str) -> Result<String, nepl_core::codegen_llvm::LlvmCodegenError> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("char_llvm_test.nepl".into(), src.to_string())
        .expect("load");
    nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
}

#[test]
fn lexer_accepts_char_literals_and_escapes() {
    let src = "'a' '\\n' '\\b' '\\f' '\\'' '\\\\' '\\x41' '\\u{3042}'";
    let lexed = lexer::lex(FileId(0), src);
    assert!(
        lexed
            .diagnostics
            .iter()
            .all(|d| !matches!(d.severity, Severity::Error)),
        "unexpected lexer errors: {:?}",
        lexed.diagnostics
    );
    let chars: Vec<u32> = lexed
        .tokens
        .iter()
        .filter_map(|token| match token.kind {
            TokenKind::CharLiteral(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(
        chars,
        vec![
            'a' as u32,
            '\n' as u32,
            '\u{08}' as u32,
            '\u{0c}' as u32,
            '\'' as u32,
            '\\' as u32,
            'A' as u32,
            'あ' as u32
        ]
    );
}

#[test]
fn char_literal_runs_as_char_value() {
    let src = r#"
#entry main
fn main <()->i32> ():
    let c <char> 'A';
    match c:
        'A':
            65
        _:
            0
"#;
    assert_eq!(run_main_i32(src), 65);
}

#[test]
fn char_literal_supports_unicode_scalar_match() {
    let src = r#"
#entry main
fn main <()->i32> ():
    let c <char> '\u{3042}';
    match c:
        '\u{3042}':
            1
        _:
            0
"#;
    assert_eq!(run_main_i32(src), 1);
}

#[test]
fn char_literal_match_arm_can_match_integer_code_points() {
    let src = r#"
#import "core/math" as *

fn classify_i32 <(i32)->i32> (x):
    match x:
        'A':
            10
        _:
            0

fn classify_u8 <(u8)->i32> (x):
    match x:
        '\n':
            1
        _:
            0

#entry main
fn main <()->i32> ():
    add classify_i32 65 classify_u8 '\n'
"#;
    assert_eq!(run_main_i32(src), 11);
}

#[test]
fn backspace_and_form_feed_escapes_lower_to_code_points() {
    let src = r#"
#import "core/math" as *

#entry main
fn main <()->i32> ():
    let b <i32> '\b';
    let f <i32> '\f';
    add b f
"#;
    assert_eq!(run_main_i32(src), 20);
}

#[test]
fn char_is_copy_through_prelude_trait_impl() {
    let src = r#"
#entry main
fn main <()->i32> ():
    let c <char> 'z';
    let a <char> c;
    let b <char> c;
    match b:
        'z':
            match a:
                'z':
                    1
                _:
                    0
        _:
            0
"#;
    assert_eq!(run_main_i32(src), 1);
}

#[test]
fn char_literal_can_be_explicitly_ascribed_to_integer_literals_only() {
    let src = r#"
#import "core/math" as *

#entry main
fn main <()->i32> ():
    let a <i32> 'A';
    let b <u8> '\x02';
    add a 2
"#;
    assert_eq!(run_main_i32(src), 67);
}

#[test]
fn char_literal_uses_integer_context_for_function_arguments() {
    let src = r#"
fn takes_i32 <(i32)->i32> (x):
    x

fn takes_u8 <(u8)->u8> (x):
    x

#entry main
fn main <()->i32> ():
    let b <u8> takes_u8 '\x02';
    takes_i32 'A'
"#;
    assert_eq!(run_main_i32(src), 65);
}

#[test]
fn char_variables_do_not_implicitly_coerce_to_integer_types() {
    let src = r#"
#entry main
fn main <()->i32> ():
    let c <char> 'A';
    let n <i32> c;
    n
"#;
    assert!(compile_with_loader(src).is_err());
}

#[test]
fn char_cast_intrinsics_emit_llvm_as_i32_noops() {
    let src = r#"
#entry main
#target llvm
fn from_code_raw <(i32)->char> (v):
    #intrinsic "i32_to_char" <> (v)

fn to_code_raw <(char)->i32> (c):
    #intrinsic "char_to_i32" <> (c)

fn main <()->i32> ():
    let c <char> from_code_raw 65;
    to_code_raw c
"#;
    let ll = emit_llvm_with_loader(src).expect("char cast intrinsics should emit LLVM IR");
    assert!(ll.contains("define i32"));
    assert!(ll.contains("from_code_raw"));
    assert!(ll.contains("to_code_raw"));
    assert!(!ll.contains("char_to_i32"));
    assert!(!ll.contains("i32_to_char"));
}

#[test]
fn lexer_rejects_multi_character_char_literal() {
    let lexed = lexer::lex(FileId(0), "'ab'");
    assert!(
        lexed
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Error)),
        "expected lexer error, got {:?}",
        lexed.diagnostics
    );
}

#[test]
fn char_match_rejects_integer_arm() {
    let src = r#"
#entry main
fn main <()->i32> ():
    let c <char> 'A';
    match c:
        65:
            1
        _:
            0
"#;
    assert!(compile_with_loader(src).is_err());
}
