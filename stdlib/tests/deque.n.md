# stdlib/deque.n.md

## deque_push_front_back

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"deque_push_front_back\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"deque len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"front item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"back item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let dq0 %Deque i32:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_front 5 |> uwok
        |> push_back 20 |> uwok
    let size %i32 len<i32> &dq0;
    free<i32> dq0;
    let dq1 %Deque i32:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_front 5 |> uwok
    let ok1 %bool match peek_front<i32> &dq1:
        Option::Some v:
            eq v 5
        Option::None:
            false
    free<i32> dq1;
    let dq2 %Deque i32:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let ok2 %bool match peek_back<i32> &dq2:
        Option::Some v:
            eq v 20
        Option::None:
            false
    free<i32> dq2;
    let report:
        test_report_new "deque_push_front_back"
        |> test_report_push assert_eq_i32 "deque len" 3 size
        |> test_report_push assert "front item" ok1
        |> test_report_push assert "back item" ok2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## deque_pop_both_ends

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"deque_pop_both_ends\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"pop_front returns front\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"pop_front keeps remaining owner\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"pop_back returns back\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"pop_back keeps remaining owner\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let dq_front %Deque i32:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let p_front %DequePop i32 pop_front<i32> dq_front
    let ok0 %bool match deque_pop_item<i32> &p_front:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let dq_front_next %Deque i32 deque_pop_deque<i32> p_front
    let len_front_next %i32 len<i32> &dq_front_next
    free<i32> dq_front_next
    let dq_back %Deque i32:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let p_back %DequePop i32 pop_back<i32> dq_back
    let ok1 %bool match deque_pop_item<i32> &p_back:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let dq_back_next %Deque i32 deque_pop_deque<i32> p_back
    let len_back_next %i32 len<i32> &dq_back_next
    free<i32> dq_back_next
    let report:
        test_report_new "deque_pop_both_ends"
        |> test_report_push assert "pop_front returns front" ok0
        |> test_report_push assert_eq_i32 "pop_front keeps remaining owner" 1 len_front_next
        |> test_report_push assert "pop_back returns back" ok1
        |> test_report_push assert_eq_i32 "pop_back keeps remaining owner" 1 len_back_next
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
