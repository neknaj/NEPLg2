# 型で失敗を表す（Option / Result の徹底）

型駆動スタイルでは「失敗を例外に逃がさず、型で明示する」流れが重要です。
NEPLg2 でも `Option` と `Result` を使うと、呼び出し側で分岐を強制できるため安全です。

## `Result` でエラー理由を持たせる

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn checked_half <(i32)->Result<i32,str>> (x):
    if:
        cond eq mod_s x 2 0
        then Result::Ok i32_div_s x 2
        else Result::Err "not even"

fn main <()*> ()> ():
    match checked_half 10:
        Result::Ok v:
            assert_eq_i32 5 v
        Result::Err e:
            test_fail "expected Ok"

    match checked_half 7:
        Result::Ok v:
            test_fail "expected Err"
        Result::Err e:
            assert_str_eq "not even" e

    test_checked "result as contract"
```

## `Option` で「値がない」を表現する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/option" as *
#import "std/test" as *

fn choose_positive <(i32)->Option<i32>> (x):
    if lt 0 x then some<i32> x else none<i32>

fn main <()*> ()> ():
    let a <Option<i32>> choose_positive 8
    let b <Option<i32>> choose_positive -1

    assert_eq_i32 8 option_unwrap_or<i32> a 0
    assert_eq_i32 0 option_unwrap_or<i32> b 0
    test_checked "option as explicit absence"
```
