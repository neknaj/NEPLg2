# tests/list_collections.n.md

## list_reverse_result_preserves_order

[目的/もくてき]:
- `reverse` が `Result` として[成功/せいこう]を[返/かえ]し、[逆順/ぎゃくじゅん]の[新/あたら]しい list を[作/つく]ることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `reverse`
- `Result::Ok`
- [逆順/ぎゃくじゅん] list の `get`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let src_first <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let first_ok <bool> match reverse<i32> src_first:
        Result::Err _e:
            false
        Result::Ok rev:
            match get<i32> rev 0:
                Option::Some x:
                    eq x 3
                Option::None:
                    false
    let src_last <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let last_ok <bool> match reverse<i32> src_last:
        Result::Err _e:
            false
        Result::Ok rev:
            match get<i32> rev 2:
                Option::Some x:
                    eq x 1
                Option::None:
                    false
    if and first_ok last_ok 1 0
```

## list_reverse_empty_result_is_ok

[目的/もくてき]:
- [空/から] list の `reverse` が[確保/かくほ]なしで `Ok` を[返/かえ]し、[空/から] list を[保/たも]つことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `reverse`
- [空/から] list
- `is_empty`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let empty <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    match reverse<i32> empty:
        Result::Err _e:
            0
        Result::Ok rev:
            let ok <bool> is_empty<i32> rev
            if ok 1 0
```
