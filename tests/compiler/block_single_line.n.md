# block_single_line.rs 由来の doctest

このファイルは Rust テスト `block_single_line.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## block_sl_basic_literal

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block 10
```

## block_sl_basic_arithmetic

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    block add 1 2
```

## block_sl_with_let

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block let x 10; x
```

## block_sl_multiple_stmts

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    block let x 1; let y 2; add x y
```

## block_sl_nested

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block block 5
```

## block_sl_nested_in_multiline

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block:
        block 10
```

## block_sl_arg_position

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    add 1 block 2
```

## block_sl_arg_position_complex

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    // add (block 1 (block 2)) と正しく解釈される
    add block 1 block 2
```

## block_sl_if_branch

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    // blockのルールによると if true (block 1 else (block 2)) と解釈されるため誤り
    if true block 1 else block 2
```

## block_sl_while_body

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let mut i 0
    // while lt i 5 (block set i add i 1) と解釈され、正しい
    while lt i 5 block set i add i 1
    i
```

## block_sl_semicolon_unit

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    // block returns unit, so we return 0 explicitly
    block 1;
    0
```

## block_sl_shadowing

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x 1
    let y block let x 2; x
    // y should be 2, outer x is 1
    if eq x 1 y 0
```

## block_sl_mutation

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let mut x 1
    block set x 2
    x
```

## block_sl_type_annotated

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    <i32> block 10
```

## block_sl_tuple_element


このテストは「単行ブロック（`block ...`）が式として評価され、その結果をタプル要素として扱える」ことを確認する意図です。
ただし、元のテストはタプルの旧リテラル記法 `(a, b)` と、数値フィールドアクセス `t.1` を用いていました。todo.md の方針ではこれらは廃止対象なので、
タプル生成は新記法 `Tuple:` に、要素取得は `core/field` の `get` に置き換えました。
これにより、テストの主旨（単行ブロック式の評価・タプル要素化）は維持したまま、仕様の最新方針に整合させています。

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/field" as *

fn main <()->i32> ():
    let t Tuple:
        block 1
        block 2
    get t 1
```

## block_sl_pipe_source

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    block 1 |> add 2
```

## block_sl_match_arm

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/mem" as *

enum E: A

fn main <()->i32> ():
    match E::A:
        A: block 10
```

## block_sl_trailing_comment

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block 1 // comment
```

## block_sl_empty_ish

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block ()
    0
```

## block_sl_deeply_nested

neplg2:test
ret: 99
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block block block 99
```

## block_sl_single_line_block_cannot_contain_multiline_if

neplg2:test[compile_fail]
diag_code: parser.token.unexpected
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block if:
        true
        1
        2
```
