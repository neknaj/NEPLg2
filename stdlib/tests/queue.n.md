# stdlib/queue.n.md

## queue_push_pop

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"queue_push_pop\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after push\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"peek sees first item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"pop returns pushed item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"pop_front item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"pop_front leaves next item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let q0 %Queue i32:
        unwrap_ok new<i32>
        |> push<i32> 1
        |> unwrap_ok
        |> push<i32> 2
        |> unwrap_ok
    let size0 %i32 len &q0;
    free q0;
    let q1 %Queue i32:
        unwrap_ok new<i32>
        |> push<i32> 1
        |> unwrap_ok
        |> push<i32> 2
        |> unwrap_ok
    let ok1 %bool match peek &q1:
        Option::Some v:
            eq v 1
        Option::None:
            false
    free q1;
    let q2 %Queue i32:
        unwrap_ok new<i32>
        |> push<i32> 5
        |> unwrap_ok
    let ok2 %bool match pop q2:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let q3 %Queue i32:
        unwrap_ok new<i32>
        |> push<i32> 7
        |> unwrap_ok
        |> push<i32> 8
        |> unwrap_ok
    let p0 %QueuePop i32 pop_front q3
    let ok3 %bool match queue_pop_item &p0:
        Option::Some v:
            eq v 7
        Option::None:
            false
    let q4 %Queue i32 queue_pop_queue p0
    let ok4 %bool:
        match peek &q4:
            Option::Some v:
                and eq len &q4 1 eq v 8
            Option::None:
                false
    free q4;
    let report:
        test_report_new "queue_push_pop"
        |> test_report_push assert_eq_i32 "len after push" 2 size0
        |> test_report_push assert "peek sees first item" ok1
        |> test_report_push assert "pop returns pushed item" ok2
        |> test_report_push assert "pop_front item" ok3
        |> test_report_push assert "pop_front leaves next item" ok4
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## queue_pop_empty

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"queue_pop_empty\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"pop empty returns none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let q %Queue i32 unwrap_ok new<i32>;
    let ok %bool match pop q:
        Option::Some _:
            false
        Option::None:
            true
    let report:
        test_report_new "queue_pop_empty"
        |> test_report_push assert "pop empty returns none" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
