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
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn main <()*>i32> ():
    let rb0 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok0 <bool> eq len<i32> &rb0 2;
    free<i32> rb0
    let rb1 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok1 <bool> match peek<i32> &rb1:
        Option::Some v:
            eq v 10
        Option::None:
            false
    free<i32> rb1
    let rb2 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let ok2 <bool> match pop<i32> rb2:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let rb3 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 30
        |> unwrap_ok<RingBuffer<i32>, Diag>
        |> push<i32> 40
        |> unwrap_ok<RingBuffer<i32>, Diag>
    let p0 <RingBufferPop<i32>> pop_front<i32> rb3
    let ok3 <bool> match *field::get_ref &p0 "item":
        Option::Some v:
            eq v 30
        Option::None:
            false
    let rb4 <RingBuffer<i32>> field::get p0 "buffer"
    let ok4 <bool>:
        match peek<i32> &rb4:
            Option::Some v:
                and eq len<i32> &rb4 1 eq v 40
            Option::None:
                false
    free<i32> rb4;
    if and and ok0 ok1 and ok2 and ok3 ok4 1 0
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
