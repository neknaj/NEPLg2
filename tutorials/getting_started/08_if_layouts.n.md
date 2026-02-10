# if の書式バリエーション

NEPLg2 の `if` は、同じ意味を保ったまま複数のレイアウトで書けます。
コードの長さや入れ子の深さに応じて、読みやすい形を選びます。

## 1行で書く（inline）

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn clamp_non_negative <(i32)->i32> (x):
    if lt x 0 then 0 else x

fn main <()*> ()> ():
    assert_eq_i32 0 clamp_non_negative -9
    assert_eq_i32 6 clamp_non_negative 6
    test_checked "if inline"
```

## `if:` で複数行に分ける

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn pick <(bool,i32,i32)->i32> (c, a, b):
    if:
        cond c
        then a
        else b

fn main <()*> ()> ():
    assert_eq_i32 11 pick true 11 22
    assert_eq_i32 22 pick false 11 22
    test_checked "if colon arguments"
```

## `then:` / `else:` を block として使う

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "std/test" as *
|
fn score <(i32)->i32> (n):
    if:
        cond lt n 0
        then:
            0
        else:
            add n 100

fn main <()*> ()> ():
    assert_eq_i32 0 score -1
    assert_eq_i32 107 score 7
    test_checked "if with block then else"
```
