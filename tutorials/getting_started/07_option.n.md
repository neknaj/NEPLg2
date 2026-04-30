# Option

`Option<T>` は「値がある / ない」を型で表します。`None` の可能性がある値は `match` してから使います。

neplg2:test
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
#import "std/test" as *

fn point_or_zero <(Option<i32>)->i32> (value):
    match value:
        Option::Some n:
            n
        Option::None:
            0

fn main <()*>i32> ():
    let a <Option<i32>> some<i32> 42
    let b <Option<i32>> none<i32>
    let checks:
        checks_new
        |> checks_push assert_eq_i32 42 point_or_zero a
        |> checks_push assert_eq_i32 0 point_or_zero b
        |> checks_push assert is_some some<i32> 1
        |> checks_push assert is_none none<i32>
    let shown checks_print_report checks
    checks_exit_code shown
```

入門では `unwrap` より `match` と `unwrap_or` を優先します。失敗する可能性を消してから中身を使う方が、後から大きなコードへ伸ばしやすくなります。
