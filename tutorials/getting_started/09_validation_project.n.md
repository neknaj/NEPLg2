# 小さな検証 project

入力値の検証は、最初の失敗を `Err` として返す関数に分けると扱いやすくなります。呼び出し側は `match` で検証結果を受け取ります。

neplg2:test
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
#import "core/result" as *
#import "std/test" as *

fn validate_port <(i32)->Result<i32,str>> (port):
    if:
        lt port 1
        then:
            Result<i32,str>::Err "port too small"
        else:
            if:
                lt 65535 port
                then:
                    Result<i32,str>::Err "port too large"
                else:
                    Result<i32,str>::Ok port

fn expect_port <(i32,i32)->Result<(),str>> (input, expected):
    match validate_port input:
        Result::Ok port:
            check_eq_i32 expected port
        Result::Err msg:
            Result<(),str>::Err msg

fn expect_port_error <(i32,str)->Result<(),str>> (input, expected):
    match validate_port input:
        Result::Ok port:
            Result<(),str>::Err "expected validation error"
        Result::Err msg:
            check_str_eq expected msg

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push expect_port 8080 8080
        |> checks_push expect_port_error 0 "port too small"
        |> checks_push expect_port_error 70000 "port too large"
    let shown checks_print_report checks
    checks_exit_code shown
```

validation の段階で理由付きの `Err` にしておくと、呼び出し元は panic せずに表示、既定値、再入力などを選べます。
