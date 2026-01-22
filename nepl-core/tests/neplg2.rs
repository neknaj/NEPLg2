use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

fn compile_ok(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: CompileTarget::Wasm,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: CompileTarget::Wasm,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_ok_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(FileId(0), src, CompileOptions { target });
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(FileId(0), src, CompileOptions { target });
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
fn compiles_literal_main() {
    let src = r#"
#entry main
fn main <() -> i32> ():
    #import "std/math"
    #use std::math::*
    1
"#;
    compile_ok(src);
}

#[test]
fn compiles_add_block_expression() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    #import "std/math"
    #use std::math::*
    add 1:
        add 2 3
"#;
    compile_ok(src);
}

#[test]
fn set_type_mismatch_is_error() {
    let src = r#"
#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
"#;
    compile_err(src);
}

#[test]
fn pure_cannot_call_impure() {
    let src = r#"
#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    #import "std/math"
    #use std::math::*
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
"#;
    compile_err(src);
}

#[test]
fn iftarget_non_wasm_is_skipped() {
    let src = r#"
#entry main

#if[target=other]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    compile_ok(src);
}

#[test]
fn wasm_stack_mismatch_is_error() {
    let src = r#"
#entry main

#if[target=wasm]
fn add_one <(i32)->i32> (a):
    #wasm:
        local.get $a
        // missing value for add
        i32.add

fn main <() -> i32> ():
    #import "std/math"
    #use std::math::*
    add_one 1
"#;
    compile_err(src);
}

#[test]
fn wasi_allows_wasm_gate() {
    let src = r#"
#entry main

#if[target=wasm]
fn only_wasm <() -> i32> ():
    123

fn main <() -> i32> ():
    only_wasm
"#;
    compile_ok_target(src, CompileTarget::Wasi);
}

#[test]
fn wasm_skips_wasi_gate() {
    let src = r#"
#entry main

#if[target=wasi]
fn only_wasi <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
"#;
    compile_ok_target(src, CompileTarget::Wasm);
    compile_err_target(src, CompileTarget::Wasi);
}

#[test]
fn string_literal_compiles() {
    let src = r#"
#entry main
#indent 4
#extern "env" "print_str" fn print_str <(str)*>()>

fn main <()*> ()> ():
    print_str "hello";
    ()
"#;
    compile_ok(src);
}
