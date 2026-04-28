# Option

`Option<T>` は「値がある / ない」を型で表します。`None` の可能性がある値は `match` してから使います。

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

fn point_or_zero <(Option<i32>)->i32> (value):
    match value:
        Option::Some n:
            n
        Option::None:
            0

fn main <()*>i32> ():
    let a <Option<i32>> some<i32> 42
    let b <Option<i32>> none<i32>
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 42 point_or_zero a
        |> checks_push check_eq_i32 0 point_or_zero b
        |> checks_push check is_some some<i32> 1
        |> checks_push check is_none none<i32>
    checks_exit_code checks
```

入門では `unwrap` より `match` と `unwrap_or` を優先します。失敗する可能性を消してから中身を使う方が、後から大きなコードへ伸ばしやすくなります。
