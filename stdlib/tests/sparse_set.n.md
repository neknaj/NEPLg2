# stdlib/sparse_set.n.md

## sparse_set_insert_remove_and_membership

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 10
        |> insert 2 |> uwok
        |> insert 4 |> uwok
        |> insert 7 |> uwok
        |> remove 4 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains &s 2;
    let ok1 <bool> not unwrap_ok<bool, Diag> contains &s 4;
    let ok2 <bool> eq len &s 2;
    let ok3 <bool> eq universe_len &s 10;
    free s
    if and and ok0 ok1 and ok2 ok3 1 0
```

## sparse_set_invalid_index

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let s0 <SparseSet> unwrap_ok<SparseSet, Diag> new 6;
    let r0 <Result<bool, Diag>> contains &s0 8;
    let s1 <SparseSet> unwrap_ok<SparseSet, Diag> new 6;
    let ok1 <bool> match insert s1 8:
        Result::Ok bad:
            free bad
            false
        Result::Err e:
            let _d <Diag> sparse_set_update_error_diag &e
            let recovered <SparseSet> sparse_set_update_error_owner e
            free recovered
            true
    let s2 <SparseSet> unwrap_ok<SparseSet, Diag> new 6;
    let ok2 <bool> match remove s2 8:
        Result::Ok bad:
            free bad
            false
        Result::Err e:
            let _d <Diag> sparse_set_update_error_diag &e
            let recovered <SparseSet> sparse_set_update_error_owner e
            free recovered
            true
    let ok0 <bool> is_err<bool, Diag> r0;
    free s0
    if and ok0 and ok1 ok2 1 0
```
