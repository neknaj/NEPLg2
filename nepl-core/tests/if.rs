mod harness;
use harness::run_main_i32;

#[test]
fn if_a_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let a <i32> if true 0 1;
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_b_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let b <i32> if true then 0 else 1;
    b
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_c_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let c <i32> if:
        true
        0
        1
    c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_d_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let d <i32> if:
        cond true
        then 0
        else 1
    d
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_e_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let e <i32> if:
        true
        then:
            0
        else:
            1
    e
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_f_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let f <i32> if true 0 if true 1 2;
    f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_c_variant_lt_condition() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 1 2
        10
        20
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn if_c_variant_block_values() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        add 1 2
        add 3 4
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn if_c_variant_cond_keyword() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 3
        7
        8
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn if_mixed_cond_then_block_else_block() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then:
            11
        else:
            12
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn if_mixed_layout_then_inline_else() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            21
        else 22
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 21);
}

#[test]
fn if_mixed_cond_inline_then_block_else_inline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 1 2
        then:
            31
        else 32
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 31);
}

#[test]
fn if_inline_false_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let x <i32> if false 0 1;
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn if_block_false_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let x <i32> if:
        false
        100
        200
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 200);
}

#[test]
fn if_mixed_cond_false_then_block_else_inline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let x <i32> if:
        cond lt 2 1
        then:
            55
        else 66
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 66);
}