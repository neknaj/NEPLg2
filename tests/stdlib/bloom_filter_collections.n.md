# tests/bloom_filter_collections.n.md

## bloom_filter_pipe_usage

[目的/もくてき]:
- `BloomFilter` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `contains`
- `clear`

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
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 128
        |> insert 3
        |> insert 8
        |> insert 21
    let ok0 <bool> contains bf0 8;
    let bf1 <BloomFilter<i32, DefaultHash32>>:
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 128
        |> insert 3
        |> insert 8
        |> insert 21
        |> clear
    let ok1 <bool> not contains bf1 8;
    if and ok0 ok1 1 0
```

## bloom_filter_free_releases_owned_storage

[目的/もくてき]:
- `BloomFilter.free` が owner [管理/かんり]している bit array を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

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

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bf0 <BloomFilter<i32, DefaultHash32>>:
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 128
        |> insert 8
    free bf0
    let bf1 <BloomFilter<i32, DefaultHash32>>:
        unwrap_ok<BloomFilter<i32, DefaultHash32>, Diag> new DefaultHash32 128
        |> insert 21
    free bf1
    1
```
