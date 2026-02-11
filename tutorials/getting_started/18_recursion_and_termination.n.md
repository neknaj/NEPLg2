# [再帰/さいき]と[停止/ていし][条件/じょうけん]

NEPLg2 でも、関数は自分自身を呼び出せます。
ただし再帰では、必ず停止条件（base case）を先に明確にします。

## `sum_to` を再帰で実装する

`sum_to n` は `1 + 2 + ... + n` を返します。
`n <= 0` を停止条件にして、無限再帰を防ぎます。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn sum_to <(i32)->i32> (n):
    if:
        cond le n 0
        then 0
        else:
            let prev <i32> sub n 1
            add n sum_to prev
|
fn main <()*>()> ():
    assert_eq_i32 0 sum_to 0
    assert_eq_i32 1 sum_to 1
    assert_eq_i32 15 sum_to 5
    test_checked "recursion with base case"
```

## 条件分岐を `if:` で分けて書く

処理が長くなる場合は `if:` 形式にすると読みやすくなります。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn fib <(i32)->i32> (n):
    if:
        cond le n 1
        then n
        else:
            let n1 <i32> sub n 1
            let n2 <i32> sub n 2
            let a <i32> fib n1
            let b <i32> fib n2
            add a b
|
fn main <()*>()> ():
    assert_eq_i32 0 fib 0
    assert_eq_i32 1 fib 1
    assert_eq_i32 8 fib 6
    test_checked "recursive if-colon form"
```

## 実装上の注意

- 停止条件がない再帰は、実行時に停止しません。
- 再帰を深くしすぎるとスタックを使い切ることがあります。
- 深いループ処理は `while` に切り替える設計も検討します。
