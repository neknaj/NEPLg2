# stdlib/bloom_filter.n.md

## bloom_filter_insert_and_contains

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bf <BloomFilter<i32, DefaultHash32>>:
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 4
        |> insert 9
        |> insert 15
    let ok0 <bool> contains &bf 9;
    let ok1 <bool> eq len &bf 64;
    free bf
    if and ok0 ok1 1 0
```

## bloom_filter_clear_and_invalid_len

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bf0 <BloomFilter<i32, DefaultHash32>>:
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 64
        |> insert 7
    let bf1 <BloomFilter<i32, DefaultHash32>> clear bf0;
    let seen <bool> contains<i32, DefaultHash32> &bf1 7;
    free bf1
    let ok0 <bool> if seen false true;
    let bad <Result<BloomFilter<i32, DefaultHash32>, Diag>> new DefaultHash32 0;
    let ok1 <bool> is_err<BloomFilter<i32, DefaultHash32>, Diag> bad;
    if and ok0 ok1 1 0
```
