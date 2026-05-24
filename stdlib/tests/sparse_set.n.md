# stdlib/sparse_set.n.md

## sparse_set_insert_remove_and_membership

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sparse_set_insert_remove_and_membership\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"contains inserted value\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed value absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"sparse set len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"universe len\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let s %SparseSet:
        unwrap_ok<SparseSet, Diag> new 10
        |> insert 2 |> uwok
        |> insert 4 |> uwok
        |> insert 7 |> uwok
        |> remove 4 |> uwok
    let ok0 %bool unwrap_ok<bool, Diag> contains &s 2;
    let ok1 %bool not unwrap_ok<bool, Diag> contains &s 4;
    let size %i32 len &s;
    let universe %i32 universe_len &s;
    free s
    let report:
        test_report_new "sparse_set_insert_remove_and_membership"
        |> test_report_push assert "contains inserted value" ok0
        |> test_report_push assert "removed value absent" ok1
        |> test_report_push assert_eq_i32 "sparse set len" 2 size
        |> test_report_push assert_eq_i32 "universe len" 10 universe
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sparse_set_invalid_index

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sparse_set_invalid_index\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"contains invalid index errs\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"insert invalid index returns owner\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"remove invalid index returns owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let s0 %SparseSet unwrap_ok<SparseSet, Diag> new 6;
    let r0 %Result bool Diag contains &s0 8;
    let s1 %SparseSet unwrap_ok<SparseSet, Diag> new 6;
    let ok1 %bool match insert s1 8:
        Result::Ok bad:
            free bad
            false
        Result::Err e:
            let _d %Diag sparse_set_update_error_diag &e
            let recovered %SparseSet sparse_set_update_error_owner e
            free recovered
            true
    let s2 %SparseSet unwrap_ok<SparseSet, Diag> new 6;
    let ok2 %bool match remove s2 8:
        Result::Ok bad:
            free bad
            false
        Result::Err e:
            let _d %Diag sparse_set_update_error_diag &e
            let recovered %SparseSet sparse_set_update_error_owner e
            free recovered
            true
    let ok0 %bool is_err<bool, Diag> r0;
    free s0
    let report:
        test_report_new "sparse_set_invalid_index"
        |> test_report_push assert "contains invalid index errs" ok0
        |> test_report_push assert "insert invalid index returns owner" ok1
        |> test_report_push assert "remove invalid index returns owner" ok2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
