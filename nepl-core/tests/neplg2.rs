use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};
mod harness;
use harness::{compile_src_with_options, run_main_i32};

fn compile_ok(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_ok_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
        },
    );
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
#extern "env" "print_str" fn print <(str)*>()>

fn main <()*> ()> ():
    print "hello";
    ()
"#;
    compile_ok(src);
}

#[test]
fn pipe_injects_first_arg() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    add 1 add 2 3 |> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_requires_callable_target() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> 2
"#;
    compile_err(src);
}

#[test]
fn pipe_with_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_with_double_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_target_missing_after_annotation_is_error() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> <i32> 2
"#;
    compile_err(src);
}

#[test]
fn wasi_import_rejected_on_wasm_target() {
    let src = r#"
#entry main
#indent 4
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)->i32>
fn main <()->()> ():
    ()
"#;
    compile_err_target(src, CompileTarget::Wasm);
}

#[test]
fn name_conflict_enum_fn_is_error() {
    let src = r#"
#entry main
#indent 4

enum Foo:
    A

fn Foo <()->i32> ():
    0

fn main <()->i32> ():
    Foo
"#;
    compile_err(src);
}

#[test]
fn wasm_cannot_use_stdio() {
    let src = r#"
#entry main
#indent 4
#import "std/stdio"
#use std::stdio::*

fn main <()->()> ():
    print "hi"
"#;
    compile_err_target(src, CompileTarget::Wasm);
}

#[test]
fn run_add_returns_12() {
    let src = r#"
#entry main
#indent 4
#import "std/math"
#use std::math::*

fn main <()->i32> ():
    add 10 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn match_option_some_returns_value() {
    let src = r#"
#entry main
#indent 4
#import "std/option"
#use std::option::*

fn main <()* >i32> ():
    match some 5:
        Some v:
            v
        None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn list_get_out_of_bounds_err() {
    let src = r#"
#entry main
#indent 4
#import "std/list"
#use std::list::*
#import "std/result"
#use std::result::*

fn main <()* >i32> ():
    let lst new;
    push lst 1;
    let r get lst 10;
    match r:
        Ok v:
            v
        Err e:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn non_exhaustive_match_is_error() {
    let src = r#"
#entry main
#indent 4
#import "std/option"
#use std::option::*

fn main <()->i32> ():
    match some 1:
        Some v:
            v
"#;
    compile_err(src);
}

#[test]
fn target_directive_sets_default_to_wasi() {
    let src = r#"
#target wasi
#entry main
#indent 4
#import "std/stdio"
#use std::stdio::*

fn main <()* >()> ():
    print "ok"
"#;
    let wasm = compile_src_with_options(src, CompileOptions { target: None });
    assert!(!wasm.is_empty());
}

#[test]
fn duplicate_target_directive_is_error() {
    let src = r#"
#target wasm
#target wasi
#entry main
fn main <()->i32> ():
    0
"#;
    let result = compile_wasm(FileId(0), src, CompileOptions { target: None });
    assert!(result.is_err(), "expected error, got {:?}", result);
}
