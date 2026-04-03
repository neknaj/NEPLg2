# tests/segment_tree_collections.n.md

## segment_tree_pipe_usage

[目的/もくてき]:
- `SegmentTree` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `replace`
- `add`
- `sum_range`

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
    let total <i32> unwrap_ok<i32, Diag> sum_range st 0 3;
    if eq total 7 1 0
```
