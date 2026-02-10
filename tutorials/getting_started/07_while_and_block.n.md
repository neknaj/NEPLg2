# while と block（オフサイドルール）

この章では、`while` と `block:` を使って複数式を1つの流れとして書く方法を学びます。

NEPLg2 では制御構文も式ですが、`while` は繰り返し本体を `do:` で与えると読みやすくなります。

## while の基本

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
    let mut i <i32> 0
    let mut sum <i32> 0

    while lt i 5:
        do:
            set sum add sum i
            set i add i 1

    assert_eq_i32 10 sum
    test_checked "while basic"
```

## block は「最後の式の値」を返す

`block:` は式なので、`let` の右辺にも置けます。

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
    let x <i32> block:
        let a <i32> 3
        let b <i32> 4
        add a b

    assert_eq_i32 7 x
    test_checked "block expression"
```
