# stdlib/binary_heap.n.md

## binary_heap_push_peek_pop

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
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let ok0 <bool> eq len<i32> &hp0 4;
    free<i32> hp0;
    let hp1 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
    let ok1 <bool> match peek<i32> &hp1:
        Option::Some v:
            eq v 9
        Option::None:
            false
    free<i32> hp1;
    let hp2 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
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
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, Diag> with_capacity<i32> 8;
    let ok0 <bool> is_empty<i32> &hp0;
    free<i32> hp0;
    let hp1 <BinaryHeap<i32>> unwrap_ok<BinaryHeap<i32>, Diag> new<i32>;
    let ok1 <bool> match pop hp1:
        Option::Some _:
            false
        Option::None:
            true
    if and ok0 ok1 1 0
```

## binary_heap_borrowed_reads_preserve_owner

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
    let hp <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
    let ok_len <bool> eq len<i32> &hp 3;
    let ok_peek <bool> match peek<i32> &hp:
        Option::Some v:
            eq v 9
        Option::None:
            false
    free<i32> hp;
    if and ok_len ok_peek 1 0
```

## binary_heap_pop_max_returns_owner

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
    let popped <BinaryHeapPop<i32>> pop_max<i32> hp0;
    let item <Option<i32>> *field::get_ref &popped "item";
    let hp1 <BinaryHeap<i32>> field::get popped "heap";
    let ok_item <bool> match item:
        Option::Some v:
            eq v 9
        Option::None:
            false
    let ok_len <bool> eq len<i32> &hp1 2;
    let ok_peek <bool> match peek<i32> &hp1:
        Option::Some v:
            eq v 4
        Option::None:
            false
    free<i32> hp1;
    if and ok_item and ok_len ok_peek 1 0
```

## binary_heap_grow_preserves_order

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> with_capacity<i32> 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let p0 <BinaryHeapPop<i32>> pop_max<i32> hp0;
    let item0 <Option<i32>> *field::get_ref &p0 "item";
    let hp1 <BinaryHeap<i32>> field::get p0 "heap";
    let p1 <BinaryHeapPop<i32>> pop_max<i32> hp1;
    let item1 <Option<i32>> *field::get_ref &p1 "item";
    let hp2 <BinaryHeap<i32>> field::get p1 "heap";
    let ok0 <bool> match item0:
        Option::Some v:
            eq v 9
        Option::None:
            false
    let ok1 <bool> match item1:
        Option::Some v:
            eq v 7
        Option::None:
            false
    free<i32> hp2;
    if and ok0 ok1 1 0
```
