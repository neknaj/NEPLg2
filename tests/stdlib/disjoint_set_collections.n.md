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
    let ok0 <bool> unwrap_ok<bool, Diag> same dsu0 0 3;
    if ok0 1 0
```
