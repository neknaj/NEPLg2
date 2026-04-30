# tests/ringbuffer_collections.n.md

## ringbuffer_pipe_usage

[目的/もくてき]:
- `RingBuffer` が pipe [記法/きほう]と `Result` / `Option` を[組/く]み[合/あ]わせた[基本的/きほんてき]な[使/つか]い[方/かた]で[利用/りよう]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- `len`
- `free`
- `pop`
- `uwok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let rb <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push 4
        |> uwok
        |> push 9
        |> uwok
    let ok0 <bool> eq len<i32> &rb 2;
    free<i32> rb
    let rb2 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push 4
        |> uwok
        |> push 9
        |> uwok
    let ok1 <bool> match rb2 |> pop:
        Option::Some v:
            eq v 4
        Option::None:
            false
    if and ok0 ok1 1 0
```

## ringbuffer_grow_clear_free

[目的/もくてき]:
- `RingBuffer` が capacity 1 から grow した後でも、header 更新、`clear`、`free` が trap せず[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- `push`
- grow
- `clear`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let rb_clear <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> with_capacity<i32> 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> clear
    let ok0 <bool> is_empty<i32> &rb_clear;
    free<i32> rb_clear
    let rb0 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> with_capacity<i32> 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> clear
    free<i32> rb0
    let rb1 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> with_capacity<i32> 1
        |> push 12 |> uwok
    free<i32> rb1
    if ok0 1 0
```
