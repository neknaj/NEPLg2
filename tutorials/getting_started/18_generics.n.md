# generics

型パラメータは `<.T>` のように書きます。`Option<.T>` や `Result<.T,.E>` のような標準型も generic な型です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "std/test" as *

fn identity <.T: Copy> <(.T)->.T> (x):
    x

fn main <()*>i32> ():
    let maybe <Option<i32>> identity some<i32> 7
    let answer <Result<i32,str>> identity ok<i32,str> 1
    let checks:
        checks_new
        |> checks_push assert_eq_i32 42 identity 42
        |> checks_push assert identity true
        |> checks_push assert is_some<i32> maybe
        |> checks_push assert is_ok<i32,str> answer
    let shown checks_print_report checks
    checks_exit_code shown
```

この例では `Copy` bound を付け、`i32`、`bool`、`Option<i32>`、`Result<i32,str>` のような copy できる値だけを受け取ります。所有権を持つ値まで generic に扱う場合は、move / borrow / Clone の境界を別に設計します。
