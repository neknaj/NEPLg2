# tuple_new_syntax.rs 由来の doctest

このファイルは Rust テスト `tuple_new_syntax.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## tuple_basic_i32_pair

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        10
        20
    get t 0
```

## tuple_mixed_types

neplg2:test
ret: 100
```neplg2

#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        100
        true
    if get t 1 get t 0 0
```

## tuple_nested

neplg2:test
ret: 3
```neplg2

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
```

## tuple_with_expressions

neplg2:test
ret: 8
```neplg2

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
```

## tuple_with_blocks

neplg2:test
ret: 20
```neplg2

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
```

## tuple_with_variables

neplg2:test
ret: 5
```neplg2

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
```

## tuple_as_function_arg

neplg2:test
ret: 2
```neplg2

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
```

## tuple_return_value

neplg2:test
ret: 3
```neplg2

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
```

## tuple_large

neplg2:test
ret: 6
```neplg2

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
```

## tuple_unit_elements

neplg2:test
ret: 10
```neplg2

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
```

## tuple_string_elements

neplg2:test
ret: 5
```neplg2

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
```

## tuple_struct_elements

neplg2:test
ret: 2
```neplg2

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
```

## tuple_inside_struct

neplg2:test
ret: 20
```neplg2

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
```

## tuple_generic_usage

neplg2:test
ret: 1
```neplg2

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
```

## tuple_type_annotated

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        5
        6
    get t 1
```

## tuple_multiline_expressions

neplg2:test
ret: 1
```neplg2

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
```

## tuple_with_comments

neplg2:test
ret: 2
```neplg2

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
```

## tuple_trailing_newline

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target wasm
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        1
        2

    get t 0
```

## tuple_destructuring_access

neplg2:test
ret: 10
```neplg2

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
```

## tuple_empty_is_unit

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t ()
    0
```
