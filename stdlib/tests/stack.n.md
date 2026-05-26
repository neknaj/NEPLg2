# stdlib/stack.n.md

## stack_new_and_len

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_new_and_len\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after pushes\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut s %Stack i32 unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 20;
    let stack_len %i32 len<i32> &s;
    free<i32> s;
    let report:
        test_report_new "stack_new_and_len"
        |> test_report_push assert_eq_i32 "len after pushes" 2 stack_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_peek_and_pop

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_peek_and_pop\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"peek top\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"pop top\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s0 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let ok0 %bool match peek<i32> &s0:
        Option::Some v:
            eq v 20
        Option::None:
            false
    free<i32> s0;
    let s1 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let p pop<i32> s1;
    let ok1 %bool match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let report:
        test_report_new "stack_peek_and_pop"
        |> test_report_push assert "peek top" ok0
        |> test_report_push assert "pop top" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_pop_empty

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_pop_empty\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"pop empty none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s %Stack i32 unwrap_ok<Stack<i32>, Diag> new<i32>;
    let p pop<i32> s;
    let is_empty_pop %bool match p:
        Option::Some _:
            false
        Option::None:
            true
    let report:
        test_report_new "stack_pop_empty"
        |> test_report_push assert "pop empty none" is_empty_pop
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_new_and_len_pipe

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_new_and_len_pipe\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pipe len after pushes\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let stack_len %i32 len<i32> &s;
    free<i32> s;
    let report:
        test_report_new "stack_new_and_len_pipe"
        |> test_report_push assert_eq_i32 "pipe len after pushes" 2 stack_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_peek_and_pop_pipe

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_peek_and_pop_pipe\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"pipe peek top\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"pipe pop top\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s0 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let ok0 %bool match peek<i32> &s0:
        Option::Some v:
            eq v 20
        Option::None:
            false
    free<i32> s0;
    let s1 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push<i32> 20
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let p %Option i32 pop<i32> s1;
    let ok1 %bool match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let report:
        test_report_new "stack_peek_and_pop_pipe"
        |> test_report_push assert "pipe peek top" ok0
        |> test_report_push assert "pipe pop top" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_pop_empty_pipe

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_pop_empty_pipe\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"pipe pop empty none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s %Stack i32 unwrap_ok<Stack<i32>, Diag> new<i32>;
    let p %Option i32 pop<i32> s;
    let is_empty_pop %bool match p:
        Option::Some _:
            false
        Option::None:
            true
    let report:
        test_report_new "stack_pop_empty_pipe"
        |> test_report_push assert "pipe pop empty none" is_empty_pop
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_alias_pipe_api

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_alias_pipe_api\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"alias pop top\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"alias len\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s0 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new
        |> push 1
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
        |> push 2
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let p pop s0;
    let ok0 %bool match p:
        Option::Some v:
            eq v 2
        Option::None:
            false
    let s1 %Stack i32:
        unwrap_ok<Stack<i32>, Diag> new
        |> push 5
        |> unwrap_ok<Stack<i32>, StackPushError<i32>>
    let s1_len %i32 len &s1;
    free s1;
    let report:
        test_report_new "stack_alias_pipe_api"
        |> test_report_push assert "alias pop top" ok0
        |> test_report_push assert_eq_i32 "alias len" 1 s1_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_get_keeps_stack

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_get_keeps_stack\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"get index 0\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"len before push\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"len after push\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut s %Stack i32 unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 20;
    let first_ok %bool match get<i32> &s 0:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let len_before %i32 len<i32> &s;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 30;
    let len_after %i32 len<i32> &s;
    free<i32> s;
    let report:
        test_report_new "stack_get_keeps_stack"
        |> test_report_push assert "get index 0" first_ok
        |> test_report_push assert_eq_i32 "len before push" 2 len_before
        |> test_report_push assert_eq_i32 "len after push" 3 len_after
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stack_pop_top_keeps_stack

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"stack_pop_top_keeps_stack\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"first pop top\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"second pop top\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"empty after pops\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"len after repush\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut s %Stack i32 unwrap_ok<Stack<i32>, Diag> new<i32>;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 10;
    set s unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s 20;
    let p0 %StackPop i32 pop_top<i32> s;
    let a %Option i32 stack_pop_item<i32> &p0;
    let s1 %Stack i32 stack_pop_stack<i32> p0;
    let p1 %StackPop i32 pop_top<i32> s1;
    let b %Option i32 stack_pop_item<i32> &p1;
    let s2 %Stack i32 stack_pop_stack<i32> p1;
    let empty_len %i32 len<i32> &s2;
    let s3 %Stack i32 unwrap_ok<Stack<i32>, StackPushError<i32>> push<i32> s2 30;
    let repush_len %i32 len<i32> &s3;
    let a_ok %bool match a:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let b_ok %bool match b:
        Option::Some v:
            eq v 10
        Option::None:
            false
    free<i32> s3;
    let report:
        test_report_new "stack_pop_top_keeps_stack"
        |> test_report_push assert "first pop top" a_ok
        |> test_report_push assert "second pop top" b_ok
        |> test_report_push assert_eq_i32 "empty after pops" 0 empty_len
        |> test_report_push assert_eq_i32 "len after repush" 1 repush_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
