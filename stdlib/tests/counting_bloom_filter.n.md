# stdlib/counting_bloom_filter.n.md

## counting_bloom_filter_insert_remove_contains

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let bf0 <CountingBloomFilter<i32, DefaultHash32>>:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
    let ok0 <bool> contains &bf0 9;
    let ok1 <bool> eq len &bf0 64;
    free bf0
    let bf1 <CountingBloomFilter<i32, DefaultHash32>>:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
        |> remove 9
    let ok2 <bool> eq len &bf1 64;
    free bf1
    if and ok0 and ok1 ok2 1 0
```

## counting_bloom_filter_clear

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let bf0 <CountingBloomFilter<i32, DefaultHash32>>:
        unwrap_ok<CountingBloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 7
        |> clear
    let ok0 <bool> not contains &bf0 7;
    free bf0
    if ok0 1 0
```
