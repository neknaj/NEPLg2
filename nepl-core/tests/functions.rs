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
fn plus @add_nums;

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
    apply 5 @square
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
        @add_op
    else:
        sub_op
        @sub_op

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

#[test]
fn function_recursive_factorial() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn fact <(i32)->i32> (n):
    if le n 1:
        1
    else:
        mul n fact sub n 1

fn main <()->i32> ():
    fact 5
"#;
    // 5 * 4 * 3 * 2 * 1 = 120
    let v = run_main_i32(src);
    assert_eq!(v, 120);
}

#[test]
fn function_first_class_literal() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn apply <(i32, (i32)->i32)->i32> (val, func):
    func val

fn main <()->i32> ():
    // 関数リテラルを直接引数として渡す
    apply 10 (x):
        mul x 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn function_nested_capture_variable() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let y <i32> 10;

    // ネストされた関数が外側のスコープの変数 'y' をキャプチャする
    fn add_y <(i32)->i32> (x):
        add x y

    add_y 5
"#;
    // 5 + 10 = 15
    let v = run_main_i32(src);
    assert_eq!(v, 15);
}

#[test]
fn function_purity_check_pure_calls_impure() {
    let src = r#"
#entry main
#indent 4
#target wasi
#import "std/stdio" as *

// 副作用を持つ非純粋関数
fn impure_print <(i32)*>i32> (x):
    println_i32 x;
    x

// 純粋関数から非純粋関数を呼び出す (エラーになるべき)
fn pure_caller <(i32)->i32> (x):
    impure_print x

fn main <()->i32> ():
    pure_caller 1
"#;
    compile_err(src);
}

#[test]
fn function_purity_check_impure_calls_pure() {
    let src = r#"
#entry main
#indent 4
#target wasi
#import "std/stdio" as *
#import "core/math" as *

// 純粋関数
fn pure_mul <(i32, i32)->i32> (a, b):
    mul a b

// 非純粋関数から純粋関数を呼び出す (これはOK)
fn impure_caller <(i32)*>i32> (x):
    let res <i32> pure_mul x 10;
    println_i32 res;
    res

fn main <()->i32> ():
    impure_caller 5
"#;
    // このテストはコンパイルと実行が通ることを確認します。
    // 実際の出力はキャプチャしませんが、戻り値は確認できます。
    let v = run_main_i32(src);
    assert_eq!(v, 50);
}

#[test]
fn function_complex_call_precedence() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    // sub 100 (mul (inc 5) (add 2 3))
    // sub 100 (mul 6 5)
    // sub 100 30
    // => 70
    sub 100 mul inc 5 add 2 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 70);
}
