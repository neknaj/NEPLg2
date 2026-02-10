mod harness;
use harness::run_main_i32;

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

fn compile_ok(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

#[test]
fn epilogue_drop_preserves_return_value() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1;
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn match_arm_local_drop_preserves_return() {
    let src = r#"
#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some<i32> 5:
        Some v:
            let y v;
            v
        None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn trailing_semicolon_makes_block_unit_and_errors_for_return() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;
"#;
    compile_err(src);
}

#[test]
fn no_semicolons_on_line_allowed() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2
    add 3 4
    add 5 6
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn multiple_semicolons_on_line_allowed() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 1 2;;
    add 3 4;;;
    add 5 6
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}
