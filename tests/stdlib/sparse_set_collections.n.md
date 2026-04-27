# tests/sparse_set_collections.n.md

## sparse_set_pipe_usage

[目的/もくてき]:
- `SparseSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `clear`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains s0 9;
    let s1 <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
        |> clear
    let ok1 <bool> eq len s1 0;
    if and ok0 ok1 1 0
```

## sparse_set_clear_free_reallocates

[目的/もくてき]:
- `SparseSet` が insert/remove/clear 後に `free` しても trap せず、その後の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `clear`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s_free <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
        |> clear
    free s_free
    let s0 <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 7 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains s0 7;
    if ok0 1 0
```
