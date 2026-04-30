# Result

`Result<T,E>` は「成功値 / 失敗理由」を型で表します。失敗しうる関数は `Result` を返し、呼び出し側が `match` で成功と失敗を分けます。

neplg2:test
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn divide_10 <(i32)->Result<i32,str>> (x):
    if:
        eq x 0
        then:
            Result<i32,str>::Err "division by zero"
        else:
            Result<i32,str>::Ok div_s 10 x

fn expect_ok <(Result<i32,str>,i32)->Result<(),str>> (got, expected):
    match got:
        Result::Ok value:
            check_eq_i32 expected value
        Result::Err msg:
            Result<(),str>::Err msg

fn expect_err <(Result<i32,str>,str)->Result<(),str>> (got, expected):
    match got:
        Result::Ok value:
            Result<(),str>::Err "expected Err"
        Result::Err msg:
            check_str_eq expected msg

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push expect_ok divide_10 2 5
        |> checks_push expect_err divide_10 0 "division by zero"
    let shown checks_print_report checks
    checks_exit_code shown
```

`Result` の中身を取り出す関数を小さく分けると、正常系と異常系の扱いをテストしやすくなります。
