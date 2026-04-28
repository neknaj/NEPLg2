# stdlib/segment_tree.n.md

## segment_tree_set_add_and_sum

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
    let st0 <SegmentTree> unwrap_ok<SegmentTree, Diag> new 6;
    let st1 <SegmentTree> unwrap_ok<SegmentTree, Diag> replace st0 2 5;
    let st2 <SegmentTree> unwrap_ok<SegmentTree, Diag> add st1 4 3;
    let total0 <i32> unwrap_ok<i32, Diag> sum_range &st2 0 6;
    let st3 <SegmentTree> unwrap_ok<SegmentTree, Diag> new 6;
    let st4 <SegmentTree> unwrap_ok<SegmentTree, Diag> replace st3 2 5;
    let st5 <SegmentTree> unwrap_ok<SegmentTree, Diag> add st4 4 3;
    let total1 <i32> unwrap_ok<i32, Diag> sum_range &st5 2 5;
    let ok0 <bool> eq total0 8;
    let ok1 <bool> eq total1 8;
    if and ok0 ok1 1 0
```

## segment_tree_invalid_range

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
    let st0 <SegmentTree> unwrap_ok<SegmentTree, Diag> new 4;
    let r0 <Result<SegmentTree, Diag>> replace st0 9 1;
    let st1 <SegmentTree> unwrap_ok<SegmentTree, Diag> new 4;
    let r1 <Result<i32, Diag>> sum_range &st1 3 1;
    let ok0 <bool> is_err<SegmentTree, Diag> r0;
    let ok1 <bool> is_err<i32, Diag> r1;
    if and ok0 ok1 1 0
```
