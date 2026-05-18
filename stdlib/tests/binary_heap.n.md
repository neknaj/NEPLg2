# stdlib/binary_heap.n.md

## binary_heap_push_peek_pop

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"binary_heap_push_peek_pop\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after push\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"peek sees max\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"pop returns max\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let size0 <i32> len<i32> &hp0;
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
    let report:
        test_report_new "binary_heap_push_peek_pop"
        |> test_report_push assert_eq_i32 "len after push" 4 size0
        |> test_report_push assert "peek sees max" ok1
        |> test_report_push assert "pop returns max" ok2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## binary_heap_empty_and_capacity

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"binary_heap_empty_and_capacity\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"with_capacity heap is empty\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty pop returns none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

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
    let report:
        test_report_new "binary_heap_empty_and_capacity"
        |> test_report_push assert "with_capacity heap is empty" ok0
        |> test_report_push assert "empty pop returns none" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## binary_heap_borrowed_reads_preserve_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"binary_heap_borrowed_reads_preserve_owner\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"borrowed len sees live heap\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"borrowed peek sees max\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

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
    let report:
        test_report_new "binary_heap_borrowed_reads_preserve_owner"
        |> test_report_push assert "borrowed len sees live heap" ok_len
        |> test_report_push assert "borrowed peek sees max" ok_peek
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## binary_heap_pop_max_returns_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"binary_heap_pop_max_returns_owner\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"pop_max item is max\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"pop_max returns shortened heap\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"remaining heap next max\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> new<i32>
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
    let popped <BinaryHeapPop<i32>> pop_max<i32> hp0;
    let item <Option<i32>> binary_heap_pop_item<i32> &popped;
    let hp1 <BinaryHeap<i32>> binary_heap_pop_heap<i32> popped;
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
    let report:
        test_report_new "binary_heap_pop_max_returns_owner"
        |> test_report_push assert "pop_max item is max" ok_item
        |> test_report_push assert "pop_max returns shortened heap" ok_len
        |> test_report_push assert "remaining heap next max" ok_peek
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## binary_heap_grow_preserves_order

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"binary_heap_grow_preserves_order\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"first pop after grow\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"second pop after grow\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, Diag> with_capacity<i32> 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> push 1 |> uwok
        |> push 7 |> uwok
    let p0 <BinaryHeapPop<i32>> pop_max<i32> hp0;
    let item0 <Option<i32>> binary_heap_pop_item<i32> &p0;
    let hp1 <BinaryHeap<i32>> binary_heap_pop_heap<i32> p0;
    let p1 <BinaryHeapPop<i32>> pop_max<i32> hp1;
    let item1 <Option<i32>> binary_heap_pop_item<i32> &p1;
    let hp2 <BinaryHeap<i32>> binary_heap_pop_heap<i32> p1;
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
    let report:
        test_report_new "binary_heap_grow_preserves_order"
        |> test_report_push assert "first pop after grow" ok0
        |> test_report_push assert "second pop after grow" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
