# 数値と変数

NEPLg2 では演算も前置記法で書きます。
`core/math` を import すると `add` や `i32_add` などを利用できます。

## 前置記法の基本

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    assert_eq_i32 6 add 1 5
    assert_eq_i32 5 sub 8 3
    assert_eq_i32 42 mul 6 7
    assert_eq_i32 4 i32_div_s 9 2
    test_checked "prefix arithmetic"
```

## 変数定義（`let`）と型注釈（`<T>`）

`let name <type> expr` の形で定義できます。
型注釈 `<i32>` は式に前置される点が NEPLg2 の特徴です。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let a <i32> 10
    let b <i32> 32
    let c <i32> add a b
    assert_eq_i32 42 c
    test_checked "let with type annotation"
```

## 可変変数（`let mut` / `set`）

`let mut` で再代入可能な変数を作り、`set` で更新します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let mut x <i32> 1
    set x add x 4
    set x mul x 3
    assert_eq_i32 15 x
    test_checked "let mut and set"
```

## 注意: `i32` のオーバーフロー

Wasm の `i32` 演算は wrap-around です。
`2147483647 + 1` は `-2147483648` になります。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let x <i32> 2147483647
    let y <i32> i32_add x 1
    assert_eq_i32 -2147483648 y
    test_checked "overflow"
```
