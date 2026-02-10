mod harness;
use harness::run_main_i32;

#[test]
fn block_sl_basic_literal() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn block_sl_basic_arithmetic() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    block add 1 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn block_sl_with_let() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block let x 10; x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn block_sl_multiple_stmts() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    block let x 1; let y 2; add x y
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn block_sl_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block block 5
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn block_sl_nested_in_multiline() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block:
        block 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn block_sl_arg_position() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 block 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn block_sl_arg_position_complex() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    // add (block 1 (block 2)) と正しく解釈される
    add block 1 block 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn block_sl_if_branch() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    // blockのルールによると if true (block 1 else (block 2)) と解釈されるため誤り
    if true block 1 else block 2
"#;
    // let v = run_main_i32(src);
    // assert_eq!(v, 1);
}

#[test]
fn block_sl_while_body() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let mut i 0
    // while lt i 5 (block set i add i 1) と解釈され、正しい
    while lt i 5 block set i add i 1
    i
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn block_sl_semicolon_unit() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    // block returns unit, so we return 0 explicitly
    block 1;
    0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn block_sl_shadowing() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x 1
    let y block let x 2; x
    // y should be 2, outer x is 1
    if eq x 1 y 0
"#;
    // if x==1 then y else 0 -> 2
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn block_sl_mutation() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let mut x 1
    block set x 2
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn block_sl_type_annotated() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    <i32> block 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn block_sl_tuple_element() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t (block 1, block 2) // Tuple 旧記法 Tuple新記法実装後は新記法に移行する必要
    t.1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn block_sl_pipe_source() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    block 1 |> add 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn block_sl_match_arm() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum E: A

fn main <()->i32> ():
    match E::A:
        A: block 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn block_sl_trailing_comment() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block 1 // comment
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn block_sl_empty_ish() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block ()
    0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn block_sl_deeply_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block block block 99
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 99);
}
