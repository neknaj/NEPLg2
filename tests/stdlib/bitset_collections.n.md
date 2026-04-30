# tests/bitset_collections.n.md

## bitset_pipe_usage

[目的/もくてき]:
- `BitSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `fill`
- `free`

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
    let ok0 <bool> unwrap_ok<bool, Diag> contains &bs0 3;
    let ok1 <bool> not unwrap_ok<bool, Diag> contains &bs0 8;
    let ok2 <bool> eq len &bs0 24;
    free bs0
    let bs2 <BitSet> fill unwrap_ok<BitSet, Diag> new 24;
    let ok3 <bool> unwrap_ok<bool, Diag> contains &bs2 8;
    free bs2
    if and ok0 and ok1 and ok2 ok3 1 0
```

## bitset_free_releases_owned_storage

[目的/もくてき]:
- `BitSet.free` が owner [管理/かんり]している bit storage を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `insert`

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
        |> insert 5 |> uwok
    free bs0
    let bs1 <BitSet>:
        unwrap_ok<BitSet, Diag> new 24
        |> insert 6 |> uwok
    free bs1
    1
```
