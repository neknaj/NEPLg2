# 関数

NEPL は `fn` で関数を定義します。型は `<(引数)->戻り値>` の形で書けます。

ここでは `add2` と `square` を作って動作を確認します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn add2 <(i32,i32)->i32> (a, b):
    i32_add a b

fn square <(i32)->i32> (x):
    i32_mul x x

fn main <()*> ()> ():
    assert_eq_i32 7 add2 3 4
    assert_eq_i32 81 square 9
    test_checked "functions"
```

## メモ：関数呼び出しの形

`add2 3 4` のように「関数名の後ろに引数を空白で並べる」スタイルです。
他の式（例：`i32_add a b`）も同じです。
