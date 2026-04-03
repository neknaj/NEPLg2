# stdlib/bitset.n.md

## bitset_insert_remove_and_len

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
        unwrap_ok<BitSet, Diag> new 32
        |> insert 1 |> uwok
        |> insert 7 |> uwok
        |> insert 15 |> uwok
        |> remove 7 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains bs0 1;
    let bs1 <BitSet>:
        unwrap_ok<BitSet, Diag> new 32
        |> insert 1 |> uwok
        |> insert 7 |> uwok
        |> insert 15 |> uwok
        |> remove 7 |> uwok
    let ok1 <bool> not unwrap_ok<bool, Diag> contains bs1 7;
    let bs2 <BitSet>:
        unwrap_ok<BitSet, Diag> new 32
        |> insert 1 |> uwok
        |> insert 7 |> uwok
        |> insert 15 |> uwok
        |> remove 7 |> uwok
    let ok2 <bool> eq len bs2 32;
    if and ok0 and ok1 ok2 1 0
```

## bitset_clear_and_fill

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
        unwrap_ok<BitSet, Diag> new 10
        |> insert 2 |> uwok
        |> clear
    let ok0 <bool> not unwrap_ok<bool, Diag> contains bs0 2;
    let bs1 <BitSet> fill unwrap_ok<BitSet, Diag> new 10;
    let ok1 <bool> unwrap_ok<bool, Diag> contains bs1 9;
    if and ok0 ok1 1 0
```
