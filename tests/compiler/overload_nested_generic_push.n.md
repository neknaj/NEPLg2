# nested generic overload resolution for `push`

このファイルは、`Vec<Result<(),str>>` のような[入れ子/いれこ]になったジェネリクス型に対して、
`new` / `push` の `Result` [返却/へんきゃく]と overload [解決/かいけつ]が正しく動くことを確認する。

## nested_generic_push_direct

[目的/もくてき]:
- `push v r` という最短の書き方で、`Vec<Result<(),str>>` に `Result<(),str>` を追加できることを確認する。
- `push<T>` のような明示型引数に頼らず、引数型から overload が選ばれることを確認する。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/math" as *

fn main <()->i32> ():
    let v0 <Vec<Result<(),str>>> unwrap_ok new<Result<(),str>>;
    let r <Result<(),str>> Result::Ok ();
    let v1 <Vec<Result<(),str>>> uwok push v0 r;
    if eq len v1 1 1 0
```

## nested_generic_push_pipe

[目的/もくてき]:
- pipe 記法の中でも `push` が同じ overload を選べることを確認する。
- `new<Result<(),str>> |> push (Result::Ok ())` のような書き方が、collectable な test API の土台として使えることを確認する。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/math" as *

fn main <()->i32> ():
    let v <Vec<Result<(),str>>>:
        unwrap_ok new<Result<(),str>>
        |> push (Result::Ok ()) |> uwok
        |> push (Result::Err "oops") |> uwok
    if eq len v 2 1 0
```
