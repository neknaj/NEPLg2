# stdlib/disjoint_set.n.md

## disjoint_set_union_same_and_size

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
    let dsu0 <DisjointSet> unwrap_ok<DisjointSet, Diag> new 6;
    let dsu1 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu0 0 1;
    let dsu2 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu1 2 3;
    let dsu3 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu2 1 2;
    let ok0 <bool> unwrap_ok<bool, Diag> same &dsu3 0 3;
    let dsu4 <DisjointSet> unwrap_ok<DisjointSet, Diag> new 6;
    let dsu5 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu4 0 1;
    let dsu6 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu5 2 3;
    let dsu7 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu6 1 2;
    let ok1 <bool> unwrap_ok<bool, Diag> same &dsu7 0 4;
    let ok2 <bool> if ok1 false true;
    let dsu8 <DisjointSet> unwrap_ok<DisjointSet, Diag> new 6;
    let dsu9 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu8 0 1;
    let dsu10 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu9 2 3;
    let dsu11 <DisjointSet> unwrap_ok<DisjointSet, Diag> union dsu10 1 2;
    let sz <i32> unwrap_ok<i32, Diag> size &dsu11 2;
    let ok3 <bool> eq sz 4;
    if and and ok0 ok2 ok3 1 0
```

## disjoint_set_invalid_index

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
    let dsu0 <DisjointSet> unwrap_ok<DisjointSet, Diag> new 3;
    let r0 <Result<i32, Diag>> find &dsu0 5;
    let dsu1 <DisjointSet> unwrap_ok<DisjointSet, Diag> new 3;
    let r1 <Result<bool, Diag>> same &dsu1 0 4;
    let ok0 <bool> is_err<i32, Diag> r0;
    let ok1 <bool> is_err<bool, Diag> r1;
    if and ok0 ok1 1 0
```
