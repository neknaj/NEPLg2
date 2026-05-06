# tests/segment_tree_collections.n.md

## segment_tree_pipe_usage

[目的/もくてき]:
- `SegmentTree` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `len`
- `replace`
- `add`
- `sum_range`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let st <SegmentTree>:
        unwrap_ok<SegmentTree, Diag> new 5
        |> replace 0 2 |> uwok
        |> replace 2 4 |> uwok
        |> add 2 1 |> uwok
    let total <i32> unwrap_ok<i32, Diag> sum_range &st 0 3;
    let ok_len <bool> eq len &st 5;
    free st
    let ok_total <bool> eq total 7;
    if and ok_len ok_total 1 0
```

## segment_tree_update_free_reallocates

[目的/もくてき]:
- `SegmentTree` が update 後に `free` しても trap せず、その後の[再確保/さいかくほ]と query が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `replace`
- `add`
- `sum_range`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let st_free <SegmentTree>:
        unwrap_ok<SegmentTree, Diag> new 5
        |> replace 0 2 |> uwok
        |> replace 2 4 |> uwok
        |> add 2 1 |> uwok
    free st_free
    let st_empty <SegmentTree> unwrap_ok<SegmentTree, Diag> new 0;
    free st_empty
    let st0 <SegmentTree>:
        unwrap_ok<SegmentTree, Diag> new 5
        |> replace 4 6 |> uwok
        |> add 4 1 |> uwok
    let total <i32> unwrap_ok<i32, Diag> sum_range &st0 4 5;
    free st0
    if eq total 7 1 0
```

## segment_tree_update_error_returns_owner

[目的/もくてき]:
- `replace` / `add` の[範囲外/はんいがい] error が `SegmentTree` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `replace`
- `add`
- `segment_tree_update_error_owner`
- `len`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let st <SegmentTree> unwrap_ok<SegmentTree, Diag> new 4;
    match replace st 8 1:
        Result::Ok next0:
            free next0
            0
        Result::Err e0:
            let st0 <SegmentTree> segment_tree_update_error_owner e0
            let ok0 <bool> eq len &st0 4
            match add st0 9 3:
                Result::Ok next1:
                    free next1
                    0
                Result::Err e1:
                    let recovered <SegmentTree> segment_tree_update_error_owner e1
                    let ok1 <bool> eq len &recovered 4
                    free recovered
                    if and ok0 ok1 1 0
```

## segment_tree_negative_length_rejected

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

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let neg <i32> sub 0 1
    match new neg:
        Result::Ok st:
            free st
            0
        Result::Err d:
            let name <str> diag_std_error_kind_str d
            if str_eq name "CapacityExceeded" 1 0
```
