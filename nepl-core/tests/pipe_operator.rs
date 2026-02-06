mod harness;
use harness::run_main_i32;

#[test]
fn pipe_basic_call() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn id <(i32)->i32> (x): x

fn main <()->i32> ():
    1 |> id
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn pipe_basic_add() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_chain_2() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2 |> add 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn pipe_chain_3() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> add 2 |> add 3 |> add 4
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn pipe_multiline_start() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1
    |> add 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_multiline_chain() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1
    |> add 2
    |> add 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn pipe_indent_handling() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x:
        1
        |> add 2
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_arg_complex() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    6 |> sub add 2 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn pipe_source_complex() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 2 |> add 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn pipe_source_block() {
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
fn pipe_annotated_step() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    1 |> <i32> add 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_tuple_source() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn f <((i32,i32))->i32> (t): t.1

fn main <()->i32> (): // Tuple 旧記法 Tuple新記法実装後は新記法に移行する必要
    (1,2) |> f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn pipe_struct_source() {
    let src = r#"
#entry main
#indent 4
#target wasm

struct S: v <i32>
fn f <(S)->i32> (s): s.v

fn main <()->i32> ():
    S 10 |> f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn pipe_into_constructor() {
    let src = r#"
#entry main
#indent 4
#target wasm

struct S: v <i32>

fn main <()->i32> ():
    let s <S> 10 |> S
    s.v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn pipe_into_variant() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum E: V <i32>

fn main <()->i32> ():
    let e <E> 20 |> E::V
    match e:
        V v: v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn pipe_nested_pipes() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 1 |> add 2 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn pipe_in_if() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if true 1 |> add 2 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_in_match() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/mem" as *

enum E: A

fn main <()->i32> ():
    match E::A:
        A: 1 |> add 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_string() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "alloc/string" as *

fn main <()->i32> ():
    "abc" |> len
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn pipe_bool() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let b true |> not
    if b 1 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}
