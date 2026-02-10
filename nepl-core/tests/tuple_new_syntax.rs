mod harness;
use harness::run_main_i32;

#[test]
fn tuple_basic_i32_pair() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        10
        20
    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn tuple_mixed_types() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        100
        true
    if get t 1 get t 0 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 100);
}

#[test]
fn tuple_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        1
        Tuple:
            2
            3
    let inner get t 1
    get inner 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn tuple_with_expressions() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        add 1 2
        sub 10 5
    add get t 0 get t 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 8);
}

#[test]
fn tuple_with_blocks() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        block:
            let x 10
            x
        block:
            let y 20
            y
    get t 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn tuple_with_variables() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let x 5
    let y 6
    let t Tuple:
        x
        y
    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn tuple_as_function_arg() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn take <((i32,i32))->i32> (t):
    get t 1

fn main <()->i32> ():
    take Tuple:
        1
        2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn tuple_return_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn make <()->(i32,i32)> ():
    Tuple:
        3
        4

fn main <()->i32> ():
    let t make
    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn tuple_large() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        1
        2
        3
        4
        5
    add get t 0 get t 4
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn tuple_unit_elements() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        ()
        10
        ()
    get t 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn tuple_string_elements() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "alloc/string" as *
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        "hello"
        "world"
    len get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn tuple_struct_elements() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

struct S:
    val <i32>

fn main <()->i32> ():
    let t Tuple:
        S 1
        S 2
    let s get t 1
    get s "val"
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn tuple_inside_struct() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

struct Wrapper:
    pair <(i32,i32)>

fn main <()->i32> ():
    let w Wrapper Tuple:
        10
        20
    let p get w "pair"
    get p 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn tuple_generic_usage() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let t id Tuple:
        1
        2
    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn tuple_type_annotated() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t <(i32,i32)> Tuple:
        5
        6
    get t 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn tuple_multiline_expressions() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        if true:
            1
            else 0
        2
    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn tuple_with_comments() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        // first element
        1
        // second element
        2
    get t 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn tuple_trailing_newline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        1
        2

    get t 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn tuple_destructuring_access() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        10
        20
    let a get t 0
    let b get t 1
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn tuple_empty_is_unit() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t ()
    0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}
