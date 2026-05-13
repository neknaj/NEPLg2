# tests/binary_heap_collections.n.md

## binary_heap_pipe_usage

[目的/もくてき]:
- `BinaryHeap` が bare API と `Result` / `Option` を[組/く]み[合/あ]わせた pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- `peek`
- `pop`
- `uwok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 3 |> uwok
        |> push 8 |> uwok
        |> push 5 |> uwok
    let ok0 <bool> match peek<i32> &hp0:
        Option::Some v:
            eq v 8
        Option::None:
            false
    free<i32> hp0;
    let hp1 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 3 |> uwok
        |> push 8 |> uwok
        |> push 5 |> uwok
    let ok1 <bool> match pop hp1:
        Option::Some v:
            eq v 8
        Option::None:
            false
    if and ok0 ok1 1 0
```

## binary_heap_zero_capacity_free

[目的/もくてき]:
- `with_capacity 0` で[作/つく]った heap の `free` が data pointer 0 を[解放/かいほう]しようとして trap しないことを[確認/かくにん]します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, Diag> with_capacity<i32> 0;
    free<i32> hp;
    1
```

## binary_heap_push_from_zero_capacity

[目的/もくてき]:
- capacity 0 の heap が[初回/しょかい] `push` で data [領域/りょういき]を[確保/かくほ]し、通常の heap [不変条件/ふへんじょうけん]を[満/み]たすことを[確認/かくにん]します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, Diag> with_capacity<i32> 0;
    let hp1 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, Diag> push<i32> hp0 42;
    let ok <bool> match peek<i32> &hp1:
        Option::Some v:
            eq v 42
        Option::None:
            false
    free<i32> hp1;
    if ok 1 0
```
