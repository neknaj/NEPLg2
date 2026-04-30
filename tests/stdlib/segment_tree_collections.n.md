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
    let ok_len <bool> eq len &st 5;
    let total <i32> unwrap_ok<i32, Diag> sum_range &st 0 3;
    let ok_total <bool> eq total 7;
    free st
    if and ok_len ok_total 1 0
```

## segment_tree_update_free_reallocates

[目的/もくてき]:
- `SegmentTree` が update 後に `free` しても trap せず、その後の[再確保/さいかくほ]と query が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `replace`
- `add`
- `len`
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
- update [失敗/しっぱい]時に `SegmentTree` owner が `SegmentTreeUpdateError` から[回収/かいしゅう]でき、`free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `replace`
- `add`
- `update_error_tree`
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
    let replace_ok <bool> match new 3:
        Result::Err _e:
            false
        Result::Ok st0:
            match replace st0 9 1:
                Result::Ok st1:
                    free st1
                    false
                Result::Err e:
                    let recovered <SegmentTree> update_error_tree e
                    free recovered
                    true
    let add_ok <bool> match new 3:
        Result::Err _e:
            false
        Result::Ok st0:
            match add st0 9 1:
                Result::Ok st1:
                    free st1
                    false
                Result::Err e:
                    let recovered <SegmentTree> update_error_tree e
                    free recovered
                    true
    if and replace_ok add_ok 1 0
```
