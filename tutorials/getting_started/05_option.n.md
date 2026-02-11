# Option（[値/あたい]が[有/あ]る/[無/な]い）

`Option<T>` は「値が有る (`Some`) / 値が無い (`None`)」を表す型です。

NEPL では `core/option` に Option と基本操作が入っています。

## Some / None と match

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    let a <Option<i32>> some<i32> 10
    let b <Option<i32>> none<i32>

    match a:
        Option::Some v:
            assert_eq_i32 10 v
        Option::None:
            test_fail "a was None"

    match b:
        Option::Some v:
            test_fail "b was Some"
        Option::None:
            ()
    test_checked "option match"
```

## `option_unwrap_or` で既定値を使う

`unwrap` は `None` で失敗するため、入門では `option_unwrap_or` を推奨します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    let some_v <Option<i32>> some<i32> 77
    let none_v <Option<i32>> none<i32>
    assert_eq_i32 77 option_unwrap_or<i32> some_v 0
    assert_eq_i32 123 option_unwrap_or<i32> none_v 123
    test_checked "option_unwrap_or"
```
