# シャドーイング（同名識別子）解決テスト

このファイルは、同名識別子が存在する場合の名前解決規則を確認します。
特に、`todo.md` にある「ローカル束縛を import/alias より優先する」要件に対応する最小ケースを先に固定します。

## local_value_over_imported_name

neplg2:test
ret: 8
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "alloc/string" as *

fn main <()->i32> ():
    let len <i32> 7;
    add len 1
```

## nested_block_shadowing_keeps_outer_binding

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 10;
    let y <i32> block let x <i32> 3; add x 1
    add x y
```

## local_function_over_imported_function

neplg2:test
ret: 7
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add <(i32,i32)->i32> (a, b):
    sub a b

fn main <()->i32> ():
    add 10 3
```

## inner_function_over_outer_function

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn f <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    fn f <(i32)->i32> (x):
        mul x 2
    add f 3 f 4
```

## local_let_over_parameter_name

neplg2:test
ret: 12
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn calc <(i32)->i32> (x):
    let x <i32> add x 10;
    sub x 3

fn main <()->i32> ():
    calc 5
```

## branch_local_shadowing_does_not_leak

neplg2:test
ret: 11
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 10;
    let y <i32> if:
        true
        then:
            let x <i32> 1;
            x
        else:
            0
    add x y
```

## import_alias_name_shadowed_by_local_value

neplg2:test
ret: 8
```neplg2
#entry main
#indent 4
#target wasm
#import "core/result" as result
#import "core/math" as *

fn main <()->i32> ():
    let result <i32> 3;
    add result 5
```

## value_name_and_callable_name_can_coexist_currently_fails

neplg2:test[compile_fail]
ret: 10
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let add <i32> 9;
    add add 1
```

## triple_nested_shadowing_prefers_nearest

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 1;
    let y <i32> block:
        let x 2;
        let z <i32> block:
            let x 3;
            x
        add mul x 10 z
    add mul x 100 y
```

## shadowing_inside_while_does_not_escape

neplg2:test
ret: 10
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let mut i <i32> 0;
    let x <i32> 10;
    while lt i 1:
        do:
            let x <i32> 3;
            set i add i x;
    x
```

## shadowing_inside_match_arm

neplg2:test
ret: 11
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/option" as *

fn main <()->i32> ():
    let x <i32> 10;
    let y <i32> match Option::Some<i32> 1:
        Option::Some x:
            add x 10
        Option::None:
            0
    add x sub y 10
```

## imported_function_name_shadowed_by_parameter_currently_fails

neplg2:test[compile_fail]
ret: 8
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn plus_one_from <(i32)->i32> (add):
    add add 1

fn main <()->i32> ():
    plus_one_from 7
```

## local_fn_shadowing_is_lexical

neplg2:test
ret: 31
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn f <(i32)->i32> (x):
    add x 1

fn g <(i32)->i32> (x):
    fn f <(i32)->i32> (y):
        mul y 10
    f x

fn main <()->i32> ():
    add g 3 f 0
```

## shadowing_builtin_like_name_len

neplg2:test
ret: 15
```neplg2
#entry main
#indent 4
#target wasm
#import "alloc/string" as *
#import "core/math" as *

fn main <()->i32> ():
    let len <i32> 5;
    let sum <i32> add len 10;
    sum
```

## let_noshadow_rejects_shadowing

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let noshadow x <i32> 1;
    let x <i32> 2;
    add x 1
```

## fn_noshadow_rejects_shadowing

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn noshadow f <(i32)->i32> (x):
    add x 1

fn f <(i32)->i32> (x):
    add x 2

fn main <()->i32> ():
    f 0
```

## let_mut_noshadow_is_invalid

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let mut noshadow x <i32> 1;
    x
```

## std_result_noshadow_unwrap_ok

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "core/result" as *

fn main <()->i32> ():
    let unwrap_ok <i32> 1;
    unwrap_ok
```

## shadowing_import_alias_and_value_in_local_block

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4
#target wasm
#import "core/result" as result
#import "core/math" as *

fn main <()->i32> ():
    let base <i32> 1;
    let v <i32> block:
        let result <i32> 5;
        result
    add base v
```
