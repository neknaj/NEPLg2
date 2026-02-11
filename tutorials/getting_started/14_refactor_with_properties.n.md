# [等式/とうしき][的/てき]リファクタと[回帰/かいき]テスト

関数型スタイルでは「この変形で意味は同じか」を小さく検証しながら進めるのが有効です。
ここでは 2 つの実装が同じ結果を返すことを、複数ケースで固定します。

## 実装を差し替えても挙動を維持する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn sum_to_loop <(i32)*>i32> (n):
    let mut i <i32> 0
    let mut acc <i32> 0
    while lt i add n 1:
        do:
            set acc add acc i
            set i add i 1
    acc

fn sum_to_formula <(i32)->i32> (n):
    i32_div_s mul n add n 1 2

fn main <()*> ()> ():
    assert_eq_i32 sum_to_loop 0 sum_to_formula 0
    assert_eq_i32 sum_to_loop 1 sum_to_formula 1
    assert_eq_i32 sum_to_loop 10 sum_to_formula 10
    assert_eq_i32 sum_to_loop 100 sum_to_formula 100
    test_checked "refactor preserves behavior"
```

## 失敗ケースも同時に固定する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn safe_div_old <(i32,i32)->Result<i32,str>> (a, b):
    if eq b 0 then Result::Err "division by zero" else Result::Ok i32_div_s a b

fn safe_div_new <(i32,i32)->Result<i32,str>> (a, b):
    if:
        cond eq b 0
        then Result::Err "division by zero"
        else Result::Ok i32_div_s a b

fn assert_same <(i32,i32)*>()> (a, b):
    match safe_div_old a b:
        Result::Ok ov:
            match safe_div_new a b:
                Result::Ok nv:
                    assert_eq_i32 ov nv
                Result::Err ne:
                    test_fail "old=Ok new=Err"
        Result::Err oe:
            match safe_div_new a b:
                Result::Ok nv:
                    test_fail "old=Err new=Ok"
                Result::Err ne:
                    ()

fn main <()*> ()> ():
    assert_same 10 2;
    assert_same 11 3;
    assert_same 10 0;
    test_checked "refactor keeps error behavior"
```
