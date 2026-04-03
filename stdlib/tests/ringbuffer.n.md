# stdlib/ringbuffer.n.md

## ringbuffer_push_pop

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
    let rb0 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok0 <bool> eq len<i32> rb0 2;
    let rb1 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok1 <bool> match peek<i32> rb1:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let rb2 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok2 <bool> match pop<i32> rb2:
        Option::Some v:
            eq v 10
        Option::None:
            false
    if and ok0 and ok1 ok2 1 0
```

## ringbuffer_pop_empty

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let rb <RingBuffer<i32>> unwrap_ok<RingBuffer<i32>, Diag> new<i32>;
    match pop<i32> rb:
        Option::Some _:
            0
        Option::None:
            1
```
