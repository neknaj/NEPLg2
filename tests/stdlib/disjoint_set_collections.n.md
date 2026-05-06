# tests/disjoint_set_collections.n.md

## disjoint_set_pipe_usage

[目的/もくてき]:
- `DisjointSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `len`
- `union`
- `same`
- `size`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let dsu0 <DisjointSet>:
        unwrap_ok<DisjointSet, Diag> new 5
        |> union 0 1 |> uwok
        |> union 3 4 |> uwok
        |> union 1 4 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> same &dsu0 0 3;
    let ok_len <bool> eq len &dsu0 5;
    free dsu0
    let dsu1 <DisjointSet>:
        unwrap_ok<DisjointSet, Diag> new 5
        |> union 0 1 |> uwok
        |> union 3 4 |> uwok
        |> union 1 4 |> uwok
    let sz <i32> unwrap_ok<i32, Diag> size &dsu1 4;
    free dsu1
    let ok1 <bool> eq sz 4;
    let ok01 <bool> and ok0 ok1;
    let ok <bool> and ok_len ok01;
    if ok 1 0
```

## disjoint_set_union_free_reallocates

[目的/もくてき]:
- `DisjointSet` の union-by-size [更新/こうしん]と `free` が、内部の owned array cleanup で trap しないことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `union`
- `free`
- `same`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let dsu_free <DisjointSet>:
        unwrap_ok<DisjointSet, Diag> new 4
        |> union 0 1 |> uwok
        |> union 2 3 |> uwok
        |> union 1 2 |> uwok
    free dsu_free
    let dsu0 <DisjointSet>:
        unwrap_ok<DisjointSet, Diag> new 4
        |> union 0 3 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> same &dsu0 0 3;
    free dsu0
    if ok0 1 0
```

## disjoint_set_new_zero_is_empty

[目的/もくてき]:
- `new 0` が[空/から]の union-find として[成功/せいこう]し、`free` と[後続/こうぞく]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new 0`
- `len`
- empty `find` の範囲外 error
- `free`
- [再確保/さいかくほ]

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let len_ok <bool> match new 0:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok <bool> eq len &dsu 0
            free dsu
            ok
    let find_err_ok <bool> match new 0:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok <bool> match find &dsu 0:
                Result::Ok _root:
                    false
                Result::Err _e:
                    true
            free dsu
            ok
    let free_ok <bool> block:
        let empty <DisjointSet> unwrap_ok<DisjointSet, Diag> new 0
        free empty
        true
    let realloc_ok <bool> match new 1:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok <bool> match find &dsu 0:
                Result::Ok root:
                    eq root 0
                Result::Err _e:
                    false
            free dsu
            ok
    if and and len_ok find_err_ok and free_ok realloc_ok 1 0
```

## disjoint_set_union_error_returns_owner

[目的/もくてき]:
- `union` の[範囲外/はんいがい] error が `DisjointSet` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `union`
- `disjoint_set_update_error_owner`
- `len`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let dsu <DisjointSet> unwrap_ok<DisjointSet, Diag> new 4;
    match union dsu 1 9:
        Result::Ok next:
            free next
            0
        Result::Err e:
            let recovered <DisjointSet> disjoint_set_update_error_owner e
            let ok <bool> eq len &recovered 4
            free recovered
            if ok 1 0
```

## disjoint_set_negative_length_rejected

[目的/もくてき]:
- `new` が[負/ふ]の length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let neg <i32> sub 0 1
    match new neg:
        Result::Ok dsu:
            free dsu
            0
        Result::Err d:
            let name <str> diag_std_error_kind_str d
            if str_eq name "CapacityExceeded" 1 0
```
