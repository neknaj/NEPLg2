mod harness;
use harness::run_main_i32;

#[test]
fn tuple_construct_and_pass() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/mem" as *
#import "std/math" as *

fn take <((i32,bool))->i32> (t):
    7

fn main <()->i32> ():
    take (1, true)
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn tuple_generic_and_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/mem" as *
#import "std/math" as *

fn make <.A,.B> <(.A,.B)->(.A,.B)> (a,b):
    (a, b)

fn take_nested <(((i32,bool),i32))->i32> (t):
    9

fn main <()->i32> ():
    let t <(i32,bool)> make 3 true
    take_nested (t, 2)
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 9);
}
