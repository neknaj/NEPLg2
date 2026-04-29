# tests/list_collections.n.md

## list_reverse_preserves_order

[目的/もくてき]:
- `reverse` が入力 list の node owner を[再利用/さいりよう]し、[逆順/ぎゃくじゅん] list を[返/かえ]すことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `reverse`
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
    let first_rev <List<i32>> reverse<i32> src_first;
    let first_ok <bool> match get<i32> first_rev 0:
        Option::Some x:
            eq x 3
        Option::None:
            false
    let src_last <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let last_rev <List<i32>> reverse<i32> src_last;
    let last_ok <bool> match get<i32> last_rev 2:
        Option::Some x:
            eq x 1
        Option::None:
            false
    if and first_ok last_ok 1 0
```

## list_reverse_empty_is_empty

[目的/もくてき]:
- [空/から] list の `reverse` が[確保/かくほ]なしで[空/から] list を[保/たも]つことを[確認/かくにん]します。

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
    let rev <List<i32>> reverse<i32> empty;
    let ok <bool> is_empty<i32> rev
    if ok 1 0
```

## list_map_filter_return_result

[目的/もくてき]:
- `map` / `filter` が `Result` として[成功/せいこう]を[返/かえ]し、[通常/つうじょう]の[変換/へんかん]結果を[確認/かくにん]できることを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- `map`
- `filter`
- `Result::Ok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn inc <(i32)->i32> (x):
    add x 1

fn is_even <(i32)->bool> (x):
    eq rem_s x 2 0

fn main <()*>i32> ():
    let map_src <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let map_ok <bool> match map<i32,i32> map_src inc:
        Result::Err _e:
            false
        Result::Ok mapped:
            match get<i32> mapped 1:
                Option::Some x:
                    eq x 3
                Option::None:
                    false
    let filter_src <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 4 |> uwok
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let filter_ok <bool> match filter<i32> filter_src is_even:
        Result::Err _e:
            false
        Result::Ok filtered:
            eq len<i32> filtered 2
    if and map_ok filter_ok 1 0
```
