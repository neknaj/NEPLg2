# stdlib/ringbuffer.n.md

## ringbuffer_push_pop

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"ringbuffer_push_pop\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after push\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"peek sees first item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"pop returns pushed item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"pop_front item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"pop_front leaves next item\" expected=\"true\" actual=\"true\" message=\"\"\n"
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
#import "std/test" as *

fn main %impure fn () i32 \():
    let rb0 %RingBuffer i32:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
    let size0 %i32 len<i32> &rb0;
    free<i32> rb0
    let rb1 %RingBuffer i32:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
    let ok1 %bool match peek<i32> &rb1:
        Option::Some v:
            eq v 10
        Option::None:
            false
    free<i32> rb1
    let rb2 %RingBuffer i32:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
    let ok2 %bool match pop<i32> rb2:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let rb3 %RingBuffer i32:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push<i32> 30
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
        |> push<i32> 40
        |> unwrap_ok<RingBuffer<i32>, RingBufferPushError<i32>>
    let p0 %RingBufferPop i32 pop_front<i32> rb3
    let ok3 %bool match ringbuffer_pop_item<i32> &p0:
        Option::Some v:
            eq v 30
        Option::None:
            false
    let rb4 %RingBuffer i32 ringbuffer_pop_buffer<i32> p0
    let ok4 %bool:
        match peek<i32> &rb4:
            Option::Some v:
                and eq len<i32> &rb4 1 eq v 40
            Option::None:
                false
    free<i32> rb4;
    let report:
        test_report_new "ringbuffer_push_pop"
        |> test_report_push assert_eq_i32 "len after push" 2 size0
        |> test_report_push assert "peek sees first item" ok1
        |> test_report_push assert "pop returns pushed item" ok2
        |> test_report_push assert "pop_front item" ok3
        |> test_report_push assert "pop_front leaves next item" ok4
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## ringbuffer_pop_empty

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"ringbuffer_pop_empty\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"pop empty returns none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let rb %RingBuffer i32 unwrap_ok<RingBuffer<i32>, Diag> new<i32>;
    let ok %bool match pop<i32> rb:
        Option::Some _:
            false
        Option::None:
            true
    let report:
        test_report_new "ringbuffer_pop_empty"
        |> test_report_push assert "pop empty returns none" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
