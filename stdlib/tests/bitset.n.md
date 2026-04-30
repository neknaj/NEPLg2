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
    let bs <BitSet>:
        unwrap_ok<BitSet, Diag> new 32
        |> insert 1 |> uwok
        |> insert 7 |> uwok
        |> insert 15 |> uwok
        |> remove 7 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains &bs 1;
    let ok1 <bool> not unwrap_ok<bool, Diag> contains &bs 7;
    let ok2 <bool> eq len &bs 32;
    free bs
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
    let ok0 <bool> not unwrap_ok<bool, Diag> contains &bs0 2;
    free bs0
    let bs1 <BitSet> fill unwrap_ok<BitSet, Diag> new 10;
    let ok1 <bool> unwrap_ok<bool, Diag> contains &bs1 9;
    free bs1
    if and ok0 ok1 1 0
```

## bitset_update_error_returns_owner

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
    let bs0 <BitSet> unwrap_ok<BitSet, Diag> new 12;
    let ok0 <bool>:
        match insert bs0 99:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <BitSet> bitset_update_error_owner e
                let ok <bool> eq len &recovered 12
                free recovered
                ok
    let bs1 <BitSet> unwrap_ok<BitSet, Diag> new 12;
    let ok1 <bool>:
        match remove bs1 sub 0 1:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <BitSet> bitset_update_error_owner e
                let ok <bool> eq len &recovered 12
                free recovered
                ok
    if and ok0 ok1 1 0
```
