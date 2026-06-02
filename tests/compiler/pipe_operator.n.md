# pipe_operator.rs 由来の doctest

このファイルは Rust テスト `pipe_operator.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## pipe_basic_call

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core

fn id %fn i32 i32 \x: x

fn main %fn void i32 \void:
    1 |> id
```

## pipe_basic_add

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1 |> add 2
```

## pipe_chain_2

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1 |> add 2 |> add 3
```

## pipe_chain_3

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1 |> add 2 |> add 3 |> add 4
```

## pipe_multiline_start

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1
    |> add 2
```

## pipe_multiline_chain

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1
    |> add 2
    |> add 3
```

## pipe_indent_handling

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    let x:
        1
        |> add 2
    x
```

## pipe_arg_complex

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    6 |> sub add 2 3
```

## pipe_source_complex

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    add 1 2 |> add 3
```

## pipe_source_block

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    block 1 |> add 2
```

## pipe_annotated_step

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    1 |> %i32 add 2
```

## pipe_tuple_source


このテストは「パイプ演算子 `|>` の左辺に複数行式（ここでは `Tuple:`）を置けること」と、
「`|>` の手前で改行してもインデントを増やさない」という plan.md の規則を同時に確認する意図です。
旧タプルリテラル `(1,2)` と `t.1` は todo.md で廃止対象なので、`Tuple:` と `core/field` の `get` へ置換しました。
`Tuple:` の要素行は 1 段深いインデントにし、`|>` 行は元のインデントへ戻すことで、パイプ改行規則を満たす形にしています。

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/field" as *

fn f <.T> %fn .T i32 \t: 2

fn main %fn void i32 \void:
    Tuple:
        1
        2
    |> f
```

## pipe_struct_source

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

struct S: v %i32
fn f %fn S i32 \s: s.v

fn main %fn void i32 \void:
    S 10 |> f
```

## pipe_into_constructor

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

struct S: v %i32

fn main %fn void i32 \void:
    let s %S 10 |> S
    s.v
```

## pipe_into_variant

neplg2:test
ret: 20
```neplg2

#entry main
#indent 4
#target core
#import "core/mem" as *

enum E: V %i32

fn main %fn void i32 \void:
    let e %E 20 |> E::V
    match e:
        V v: v
```

## pipe_nested_pipes

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    add 1 |> add 2 3
```

## pipe_in_if

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    if true 1 |> add 2 0
```

## pipe_in_match

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/mem" as *

enum E: A

fn main %fn void i32 \void:
    match E::A:
        A: 1 |> add 2
```

## pipe_string

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "alloc/string" as *

fn main %fn void i32 \void:
    "abc" |> len
```

## pipe_bool

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn void i32 \void:
    let b true |> not
    if b 1 0
```

## pipe_target_nested_ascribed_call_argument

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/cast" as *
#import "core/math" as *

fn take_i64 %fn i32 fn i64 i32 \x\y:
    add x %i32 cast y

fn main %fn void i32 \void:
    1 |> take_i64 %i64 cast 2
```

## pipe_stream_writer_nested_ascribed_call_argument

neplg2:test
stdout: "2\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/cast" as *
#import "core/result" as *
#import "std/iotarget" as *
#import "std/streamio" as *

fn main %impure fn void i32 \void:
    unwrap_ok open WriteStream::Stdio
    |> writeln %i64 cast 2
    |> flush
    |> close;
    0
```
