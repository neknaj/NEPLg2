# tests/vec_collections.n.md

## vec_free_zero_and_grow_reallocates

[目的/もくてき]:
- `Vec` が `with_capacity 0` を typed empty storage として[扱/あつか]い、`free` とその[後/あと]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- typed empty storage
- `push`
- grow
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let empty <Vec<i32>> unwrap_ok with_capacity<i32> 0;
    let empty_is_empty <bool> is_empty<i32> &empty;
    let empty_cap_zero <bool> eq cap<i32> &empty 0;
    let empty_ok <bool> and empty_is_empty empty_cap_zero;
    free<i32> empty;
    let mut grown <Vec<i32>> unwrap_ok new<i32>;
    set grown unwrap_ok push<i32> grown 0;
    set grown unwrap_ok push<i32> grown 1;
    set grown unwrap_ok push<i32> grown 2;
    set grown unwrap_ok push<i32> grown 3;
    set grown unwrap_ok push<i32> grown 4;
    set grown unwrap_ok push<i32> grown 5;
    set grown unwrap_ok push<i32> grown 6;
    set grown unwrap_ok push<i32> grown 7;
    set grown unwrap_ok push<i32> grown 8;
    set grown unwrap_ok push<i32> grown 9;
    let grown_ok <bool> eq len<i32> &grown 10;
    free<i32> grown;
    let mut next <Vec<i32>> unwrap_ok new<i32>;
    set next unwrap_ok push<i32> next 42;
    let top_ok <bool> match get<i32> &next 0:
        Option::Some v:
            eq v 42
        Option::None:
            false
    free<i32> next;
    let post_ok <bool> and grown_ok top_ok;
    if and empty_ok post_ok 1 0
```

## vec_sort_merge_ret_releases_scratch_buffer

[目的/もくてき]:
- merge sort の[作業/さぎょう] buffer cleanup が trap せず、その[後/あと]の `Vec` [再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `sort_merge_ret`
- scratch buffer cleanup
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let unsorted <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 5 |> uwok
        |> push 2 |> uwok
        |> push 4 |> uwok
        |> push 1 |> uwok
    let sorted <Vec<i32>> unwrap_ok sort_merge_ret<i32> unsorted;
    let first_ok <bool> match get<i32> &sorted 0:
        Option::Some v:
            eq v 1
        Option::None:
            false
    let last_ok <bool> match get<i32> &sorted 3:
        Option::Some v:
            eq v 5
        Option::None:
            false
    free<i32> sorted;
    let mut next <Vec<i32>> unwrap_ok new<i32>;
    set next unwrap_ok push<i32> next 7;
    let next_ok <bool> match get<i32> &next 0:
        Option::Some v:
            eq v 7
        Option::None:
            false
    free<i32> next;
    if and first_ok and last_ok next_ok 1 0
```

## vec_negative_capacity_rejected

[目的/もくてき]:
- `with_capacity` が[負/ふ]の capacity を allocator に[渡/わた]さず、typed error として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- `StdErrorKind::InvalidOperation`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let neg <i32> sub 0 1
    match with_capacity<i32> neg:
        Result::Ok v:
            free<i32> v
            0
        Result::Err e:
            let name <str> std_error_kind_str e
            if str_eq name "InvalidOperation" 1 0
```
