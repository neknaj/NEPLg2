# tests/disjoint_set_collections.n.md

## disjoint_set_pipe_usage

[目的/もくてき]:
- `DisjointSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `union`
- `same`
- `size`

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
    let ok0 <bool> unwrap_ok<bool, Diag> same dsu0 0 3;
    let dsu1 <DisjointSet>:
        unwrap_ok<DisjointSet, Diag> new 5
        |> union 0 1 |> uwok
        |> union 3 4 |> uwok
        |> union 1 4 |> uwok
    let sz <i32> unwrap_ok<i32, Diag> size dsu1 4;
    let ok1 <bool> eq sz 4;
    if and ok0 ok1 1 0
```
