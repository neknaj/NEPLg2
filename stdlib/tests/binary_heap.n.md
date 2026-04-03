# stdlib/binary_heap.n.md

## binary_heap_push_peek_pop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let ok0 <bool> eq len hp0 4;
    let hp1 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
    let ok1 <bool> match peek hp1:
        Option::Some v:
            eq v 9
        Option::None:
            false
    let hp2 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let ok2 <bool> match pop hp2:
        Option::Some v:
            eq v 9
        Option::None:
            false
    if and ok0 and ok1 ok2 1 0
```

## binary_heap_empty_and_capacity

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, StdErrorKind> with_capacity<i32> 8;
    let ok0 <bool> is_empty hp0;
    let hp1 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>;
    let ok1 <bool> match pop hp1:
        Option::Some _:
            false
        Option::None:
            true
    if and ok0 ok1 1 0
```
