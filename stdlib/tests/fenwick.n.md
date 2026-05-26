# stdlib/fenwick.n.md

## fenwick_add_and_sum

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_add_and_sum\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"fenwick len\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"prefix sum 0..4\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"range sum 1..4\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let fw %Fenwick:
        unwrap_ok fw::new 5
        |> fw::add 0 1 |> uwok
        |> fw::add 1 2 |> uwok
        |> fw::add 2 3 |> uwok
        |> fw::add 3 4 |> uwok
    let size %i32 fw::len &fw;
    let prefix4 %i32 unwrap_ok fw::sum_prefix &fw 4;
    let range_1_4 %i32 unwrap_ok fw::sum_range &fw 1 4;
    fw::free fw
    let report:
        test_report_new "fenwick_add_and_sum"
        |> test_report_push assert_eq_i32 "fenwick len" 5 size
        |> test_report_push assert_eq_i32 "prefix sum 0..4" 10 prefix4
        |> test_report_push assert_eq_i32 "range sum 1..4" 9 range_1_4
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## fenwick_bounds_error

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_bounds_error\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"add error returns owner len\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let fw %Fenwick unwrap_ok fw::new 3;
    match fw::add fw 5 1:
        Result::Ok next:
            fw::free next
            0
        Result::Err e:
            let recovered %Fenwick fw::add_error_tree e
            let size %i32 fw::len &recovered
            fw::free recovered
            let report:
                test_report_new "fenwick_bounds_error"
                |> test_report_push assert_eq_i32 "add error returns owner len" 3 size
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
