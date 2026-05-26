# stdlib/option.n.md

## option_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"option_main\" count=10 failed=0\nassertion index=0 status=ok kind=bool label=\"some is_some\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"some is not none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"none is_none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"none is not some\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"unwrap some\" expected=\"99\" actual=\"99\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"unwrap_or some\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"unwrap_or none\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"and_then some\" expected=\"12\" actual=\"12\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"and_then none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=eq_i32 label=\"copy through shared reference\" expected=\"77\" actual=\"77\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std

#import "core/option" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn positive_double %fn i32 Option i32 \x:
    if gt x 0:
        then some mul x 2
        else none

fn main %impure fn unit i32 \unit:
    let some_is_some %bool is_some some 42;
    let some_is_none %bool is_none some 42;
    let none_value %Option i32 none;
    let none_is_none %bool is_none none_value;
    let none_is_some %bool is_some none_value;
    let unwrap_some %i32 unwrap some 99;
    let unwrap_or_some %i32 unwrap_or some 10 5;
    let unwrap_or_none %i32 unwrap_or none 5;
    let and_then_some %i32 unwrap and_then some 6 positive_double;
    let and_then_none %bool is_none and_then some -1 positive_double;
    let original %Option i32 some 77
    let copied %Option i32 *&original
    let copied_value %i32 unwrap copied;

    let report:
        test_report_new "option_main"
        |> test_report_push assert "some is_some" some_is_some
        |> test_report_push assert "some is not none" not some_is_none
        |> test_report_push assert "none is_none" none_is_none
        |> test_report_push assert "none is not some" not none_is_some
        |> test_report_push assert_eq_i32 "unwrap some" 99 unwrap_some
        |> test_report_push assert_eq_i32 "unwrap_or some" 10 unwrap_or_some
        |> test_report_push assert_eq_i32 "unwrap_or none" 5 unwrap_or_none
        |> test_report_push assert_eq_i32 "and_then some" 12 and_then_some
        |> test_report_push assert "and_then none" and_then_none
        |> test_report_push assert_eq_i32 "copy through shared reference" 77 copied_value
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
