# 純粋関数の合成（状態を持たない変換）

関数型スタイルでは、「状態更新より変換関数の合成を優先する」考え方がよく使われます。
NEPLg2 でも、小さな関数を組み合わせると読みやすく保守しやすいコードになります。

## 小さな変換を積み上げる

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn clamp_0_100 <(i32)->i32> (x):
    if:
        cond lt x 0
        then 0
        else:
            if:
                cond lt 100 x
                then 100
                else x

fn add_bonus <(i32)->i32> (x):
    add x 5

fn normalize_score <(i32)->i32> (raw):
    clamp_0_100 add_bonus raw

fn main <()*> ()> ():
    assert_eq_i32 0 normalize_score -20
    assert_eq_i32 55 normalize_score 50
    assert_eq_i32 100 normalize_score 99
    test_checked "pure function pipeline"
```

## `mut` を使う版と同値かをテストで固定する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn normalize_pure <(i32)->i32> (raw):
    if:
        cond lt add raw 5 0
        then 0
        else:
            if:
                cond lt 100 add raw 5
                then 100
                else add raw 5

fn normalize_mut <(i32)*>i32> (raw):
    let mut x <i32> add raw 5
    if lt x 0:
        then set x 0
        else ()
    if lt 100 x:
        then set x 100
        else ()
    x

fn main <()*> ()> ():
    assert_eq_i32 normalize_mut -20 normalize_pure -20
    assert_eq_i32 normalize_mut 40 normalize_pure 40
    assert_eq_i32 normalize_mut 120 normalize_pure 120
    test_checked "pure vs mut equivalence"
```
