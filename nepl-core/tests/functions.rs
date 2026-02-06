mod harness;
use harness::run_main_i32;

#[test]
fn function_basic_def_and_call() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    inc 41
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
#[ignore] // Nested functions are not yet fully supported in codegen
fn function_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    fn double <(i32)->i32> (x):
        mul x 2
    
    double 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn function_alias() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_nums <(i32, i32)->i32> (a, b):
    add a b

fn plus add_nums;

fn main <()->i32> ():
    plus 10 20
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn function_first_class() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn square <(i32)->i32> (x):
    mul x x

fn apply <(i32, (i32)->i32)->i32> (val, func):
    func val

fn main <()->i32> ():
    apply 5 square
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 25);
}

#[test]
fn function_return() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_op <(i32, i32)->i32> (a, b):
    add a b

fn sub_op <(i32, i32)->i32> (a, b):
    sub a b

fn get_op <(bool)->(i32, i32)->i32> (cond):
    if cond:
        add_op
    else:
        sub_op

fn main <()->i32> ():
    let f get_op true
    f 10 5
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 15);
}

#[test]
fn function_literal() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let f <(i32)->i32> (x):
        add x 1
    
    f 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn function_literal_no_args() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let f <()->i32> ():
        123
    
    f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 123);
}
