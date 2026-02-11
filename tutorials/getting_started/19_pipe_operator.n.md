# パイプ演算子 `|>`

`a |> f b` は `f a b` と同じ意味です。
左辺の値を、右辺の関数呼び出しの第1引数へ渡すため、変換の流れを左から右へ読めます。

## 基本: 左辺を第1引数へ注入する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn main <()*>()> ():
    let a <i32> 1 |> add 2
    let b <i32> add 1 add 2 3 |> add 4
    assert_eq_i32 3 a
    assert_eq_i32 10 b
    test_checked "pipe basic"
```

## 複数段の変換を連結する

式を左から順に追えるので、段階的な変換が読みやすくなります。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn main <()*>()> ():
    let v <i32> 1 |> add 2 |> add 3 |> mul 2
    assert_eq_i32 12 v
    test_checked "pipe chain"
```

## 補足

- `|>` の右辺は「呼び出し可能な式」である必要があります。
- インデントルール上、`|>` 行だけを深くしないようにします。
