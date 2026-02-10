# functions.rs 由来の doctest

このファイルは Rust テスト `functions.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## function_basic_def_and_call

neplg2:test
ret: 42
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    inc 41
```

## function_basic_def_and_call_let

fnは関数専用のlet(糖衣構文)であるから、fnの代わりにletを用いてよい

neplg2:test
ret: 42
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

let inc <(i32)->i32> (x):
    add x 1

let main <()->i32> ():
    inc 41
```

## function_basic_def_and_call_without_type_annotation

推論できるならば型注釈は不要

neplg2:test
ret: 42
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

let inc (x):
    add x 1

fn main ():
    inc 41
```

## function_nested

neplg2:test
ret: 20
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    fn double <(i32)->i32> (x):
        mul x 2
    
    double 10
```

## function_alias

neplg2:test
ret: 30
```neplg2

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
```

## function_first_class

neplg2:test
ret: 25
```neplg2

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
```

## function_return

neplg2:test
ret: 15
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_op <(i32, i32)->i32> (a, b):
    add a b

fn sub_op <(i32, i32)->i32> (a, b):
    sub a b

fn get_op <(bool)->(i32, i32)->i32> (con):
    if:
        cond con
        then:
            add_op;
            @add_op
        else:
            sub_op
            @sub_op

fn main <()->i32> ():
    let f get_op true
    f 10 5
```

## function_literal

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let f <(i32)->i32> (x):
        add x 1
    
    f 10
```

## function_literal_no_args

neplg2:test
ret: 123
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let f <()->i32> ():
        123
    
    f
```

## function_recursive_factorial

neplg2:test
ret: 120
```neplg2

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
```

## function_first_class_literal

neplg2:test
ret: 30
```neplg2

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
```

## function_nested_capture_variable

neplg2:test
ret: 15
```neplg2

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
```

## function_nested_capture_variable_let

neplg2:test
ret: 15
```neplg2

#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let y <i32> 10;

    // ネストされた関数が外側のスコープの変数 'y' をキャプチャする
    let add_y (x):
        add x y

    add_y 5
```

## function_purity_check_pure_calls_impure

neplg2:test[compile_fail]
```neplg2

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
```

## function_purity_check_impure_calls_pure

neplg2:test
ret: 50
```neplg2

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
```

## function_complex_call_precedence

neplg2:test
ret: 70
```neplg2

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
```
