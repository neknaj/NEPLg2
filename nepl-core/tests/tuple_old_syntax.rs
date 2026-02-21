mod harness;
use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
fn tuple_old_literal_call_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn take <((i32,bool))->i32> (t):
    7

fn main <()->i32> ():
    take (1, true)
"#;
    compile_err(src);
}

#[test]
fn tuple_old_literal_construct_is_rejected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn make <.A,.B> <(.A,.B)->(.A,.B)> (a,b):
    (a, b)

fn take_nested <(((i32,bool),i32))->i32> (t):
    9

fn main <()->i32> ():
    let t <(i32,bool)> make 3 true
    take_nested (t, 2)
"#;
    compile_err(src);
}
