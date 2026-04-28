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
#import "std/test" as *

fn identity <.T> <(.T)->.T> (x):
    x

fn or_default <.T> <(Option<.T>,.T)->.T> (value, default):
    match value:
        Option::Some inner:
            inner
        Option::None:
            default

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 42 identity 42
        |> checks_push check_str_eq "nepl" identity "nepl"
        |> checks_push check_eq_i32 7 or_default some<i32> 7 0
        |> checks_push check_eq_i32 9 or_default none<i32> 9
    checks_exit_code checks
```

generic 関数は型ごとの処理を共通化できます。ただし、値を複製する処理には `Copy` や `Clone` の bound が必要です。
