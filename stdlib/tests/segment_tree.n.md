# stdlib/segment_tree.n.md

## segment_tree_set_add_and_sum

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_set_add_and_sum\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"segment tree len\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"full sum\" expected=\"8\" actual=\"8\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"subrange sum\" expected=\"8\" actual=\"8\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let st0 %SegmentTree unwrap_ok new 6;
    let st1 %SegmentTree unwrap_ok replace st0 2 5;
    let st2 %SegmentTree unwrap_ok add st1 4 3;
    let total0 %i32 unwrap_ok sum_range &st2 0 6;
    let st_len %i32 len &st2;
    free st2
    let st3 %SegmentTree unwrap_ok new 6;
    let st4 %SegmentTree unwrap_ok replace st3 2 5;
    let st5 %SegmentTree unwrap_ok add st4 4 3;
    let total1 %i32 unwrap_ok sum_range &st5 2 5;
    free st5
    let report:
        test_report_new "segment_tree_set_add_and_sum"
        |> test_report_push assert_eq_i32 "segment tree len" 6 st_len
        |> test_report_push assert_eq_i32 "full sum" 8 total0
        |> test_report_push assert_eq_i32 "subrange sum" 8 total1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## segment_tree_invalid_range

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_invalid_range\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"replace error returns owner len\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"sum invalid range errs\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let st0 %SegmentTree unwrap_ok new 4;
    match replace st0 9 1:
        Result::Ok st_bad:
            free st_bad
            0
        Result::Err e0:
            let recovered %SegmentTree segment_tree_update_error_owner e0
            let recovered_len %i32 len &recovered
            free recovered
            let st1 %SegmentTree unwrap_ok new 4;
            let r1 %Result i32 Diag sum_range &st1 3 1;
            let ok1 %bool is_err r1;
            free st1
            let report:
                test_report_new "segment_tree_invalid_range"
                |> test_report_push assert_eq_i32 "replace error returns owner len" 4 recovered_len
                |> test_report_push assert "sum invalid range errs" ok1
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
