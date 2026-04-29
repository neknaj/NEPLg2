# generics

型パラメータは `<.T>` のように書きます。`Option<.T>` や `Result<.T,.E>` のような標準型も generic な型です。

neplg2:test
ret: 0
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
        |> checks_push check_eq_i32 42 identity 42
        |> checks_push check identity true
        |> checks_push check is_some<i32> maybe
        |> checks_push check is_ok<i32,str> answer
    checks_exit_code checks
```

この例では `Copy` bound を付け、`i32`、`bool`、`Option<i32>`、`Result<i32,str>` のような copy できる値だけを受け取ります。所有権を持つ値まで generic に扱う場合は、move / borrow / Clone の境界を別に設計します。
