# 数値と変数

NEPL では `i32`（32-bit 整数）や `f64`（64-bit 浮動小数）をよく使います。
`core/math` を import すると、Wasm の命令に近い演算（`i32_add` など）が使えます。

## i32 の四則演算

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
    assert_eq_i32 5 i32_add 2 3
    assert_eq_i32 1 i32_sub 3 2
    assert_eq_i32 6 i32_mul 2 3
    test_checked "basic arith"
```

## 変数（let）

`let x <i32> ...` のように、`<型>` を書いて[明示/めいじ]できます。

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
    let c <i32> i32_add a b
    assert_eq_i32 42 c
    test_checked "let"
```

## 注意：オーバーフロー

Wasm の `i32.add` などは 32-bit の[剰余/じょうよ]（wrap-around）で計算します。
（Rust の `wrapping_add` に近い[感覚/かんかく]です。）

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
    # i32 の最大値: 2147483647
    let x <i32> 2147483647
    let y <i32> i32_add x 1
    # wrap して -2147483648 になる（2^31 を超えるため）
    assert_eq_i32 -2147483648 y
    test_checked "overflow"
```
