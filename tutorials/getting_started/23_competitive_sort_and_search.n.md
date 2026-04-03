# sort と[二分探索/にぶんたんさく]の[型/かた]

競プロでは「並べる -> 探す」が基本です。  
この章では、`Vec<i32>` を `sort` し、`lower_bound` / `upper_bound` を同じデータに適用します。

## sort の基本

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 5 |> uwok
        |> push 1 |> uwok
        |> push 3 |> uwok
        |> sort_quick_ret
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check sort_is_sorted<i32> v
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

## lower_bound / upper_bound / 個数

`lower_bound` は「`x` 以上が最初に現れる位置」、  
`upper_bound` は「`x` より大きい値が最初に現れる位置」です。  
`upper - lower` で出現回数が取れます。

neplg2:test[stdio, normalize_newlines]
stdout: "1 3 2\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "kp/kpsearch" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let v_lower <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 3 |> uwok
        |> push 3 |> uwok
        |> push 7 |> uwok
        |> sort_quick_ret
    let v_upper <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 3 |> uwok
        |> push 3 |> uwok
        |> push 7 |> uwok
        |> sort_quick_ret
    let v_count <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 3 |> uwok
        |> push 3 |> uwok
        |> push 7 |> uwok
        |> sort_quick_ret
    print_i32 lower_bound_vec_i32 v_lower 2;
    print " ";
    print_i32 upper_bound_vec_i32 v_upper 3;
    print " ";
    println_i32 count_equal_range_vec_i32 v_count 3
```
