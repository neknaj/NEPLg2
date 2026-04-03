# tests/bitset_collections.n.md

## bitset_pipe_usage

[目的/もくてき]:
- `BitSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `fill`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bs0 <BitSet>:
        unwrap_ok<BitSet, Diag> new 24
        |> insert 3 |> uwok
        |> insert 8 |> uwok
        |> insert 21 |> uwok
        |> remove 8 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains bs0 3;
    let bs1 <BitSet>:
        unwrap_ok<BitSet, Diag> new 24
        |> insert 3 |> uwok
        |> insert 8 |> uwok
        |> insert 21 |> uwok
        |> remove 8 |> uwok
    let ok1 <bool> not unwrap_ok<bool, Diag> contains bs1 8;
    let bs2 <BitSet> fill unwrap_ok<BitSet, Diag> new 24;
    let ok2 <bool> unwrap_ok<bool, Diag> contains bs2 8;
    if and ok0 and ok1 ok2 1 0
```
