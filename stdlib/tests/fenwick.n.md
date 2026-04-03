# stdlib/fenwick.n.md

## fenwick_add_and_sum

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let fw_len <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 5
        |> add 0 1 |> uwok
        |> add 1 2 |> uwok
        |> add 2 3 |> uwok
        |> add 3 4 |> uwok
    let ok0 <bool> eq len fw_len 5;
    let fw_prefix <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 5
        |> add 0 1 |> uwok
        |> add 1 2 |> uwok
        |> add 2 3 |> uwok
        |> add 3 4 |> uwok
    let prefix4 <i32> unwrap_ok<i32, Diag> sum_prefix fw_prefix 4;
    let ok1 <bool> eq prefix4 10;
    let fw_range <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 5
        |> add 0 1 |> uwok
        |> add 1 2 |> uwok
        |> add 2 3 |> uwok
        |> add 3 4 |> uwok
    let range_1_4 <i32> unwrap_ok<i32, Diag> sum_range fw_range 1 4;
    let ok2 <bool> eq range_1_4 9;
    let ok01 <bool> and ok0 ok1;
    let ok012 <bool> and ok01 ok2;
    if ok012 1 0
```

## fenwick_bounds_error

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let fw <Fenwick> unwrap_ok<Fenwick, Diag> new 3;
    match add fw 5 1:
        Result::Ok _:
            0
        Result::Err _:
            1
```
