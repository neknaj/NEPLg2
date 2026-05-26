# Vec の基本

`Vec .T` は所有権を持つ growable collection です。作成や追加は失敗しうるため、`Result` を `match` して扱います。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn build_numbers %impure fn unit Result Vec i32 str \unit:
    match new<i32>:
        Result::Err _e:
            Result::Err "vec.new failed"
        Result::Ok v0:
            match push v0 10:
                Result::Err e:
                    free vec_push_error_vec e;
                    Result::Err "vec.push 10 failed"
                Result::Ok v1:
                    match push v1 20:
                        Result::Err e:
                            free vec_push_error_vec e;
                            Result::Err "vec.push 20 failed"
                        Result::Ok v2:
                            Result::Ok v2

fn expect_item %fn &Vec i32 fn i32 fn i32 Result unit str \v\idx\expected:
    match get v idx:
        Option::Some value:
            check_eq_i32 expected value
        Option::None:
            Result::Err "missing vec item"

fn main %impure fn unit i32 \unit:
    match build_numbers:
        Result::Err msg:
            let checks checks_push checks_new Result::Err msg
            let shown checks_print_report checks
            checks_exit_code shown
        Result::Ok numbers:
            let n %i32 len &numbers
            let item0 %Result unit str expect_item &numbers 0 10
            let item1 %Result unit str expect_item &numbers 1 20
            let checks:
                checks_new
                |> checks_push assert_eq_i32 2 n
                |> checks_push item0
                |> checks_push item1
            free numbers;
            let shown checks_print_report checks
            checks_exit_code shown
```

`Vec` の owner は最後に `free` します。読み取りだけなら `&numbers` を渡し、collection 本体を消費しないようにします。
