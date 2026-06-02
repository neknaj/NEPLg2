# stdlib/disjoint_set.n.md

## disjoint_set_union_same_and_size

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_union_same_and_size\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"0 and 3 connected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"disjoint set len\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"0 and 4 disconnected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"component size\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let dsu0 %DisjointSet unwrap_ok new 6;
    let dsu1 %DisjointSet unwrap_ok union dsu0 0 1;
    let dsu2 %DisjointSet unwrap_ok union dsu1 2 3;
    let dsu3 %DisjointSet unwrap_ok union dsu2 1 2;
    let ok0 %bool unwrap_ok same &dsu3 0 3;
    let dsu_len %i32 len &dsu3;
    free dsu3
    let dsu4 %DisjointSet unwrap_ok new 6;
    let dsu5 %DisjointSet unwrap_ok union dsu4 0 1;
    let dsu6 %DisjointSet unwrap_ok union dsu5 2 3;
    let dsu7 %DisjointSet unwrap_ok union dsu6 1 2;
    let ok1 %bool unwrap_ok same &dsu7 0 4;
    free dsu7
    let disconnected %bool if ok1 false true;
    let dsu8 %DisjointSet unwrap_ok new 6;
    let dsu9 %DisjointSet unwrap_ok union dsu8 0 1;
    let dsu10 %DisjointSet unwrap_ok union dsu9 2 3;
    let dsu11 %DisjointSet unwrap_ok union dsu10 1 2;
    let component_size %i32 unwrap_ok size &dsu11 2;
    free dsu11
    let report:
        test_report_new "disjoint_set_union_same_and_size"
        |> test_report_push assert "0 and 3 connected" ok0
        |> test_report_push assert_eq_i32 "disjoint set len" 6 dsu_len
        |> test_report_push assert "0 and 4 disconnected" disconnected
        |> test_report_push assert_eq_i32 "component size" 4 component_size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## disjoint_set_invalid_index

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_invalid_index\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"find invalid index errs\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"same invalid index errs\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"union error returns owner len\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let dsu0 %DisjointSet unwrap_ok new 3;
    let r0 %Result i32 Diag find &dsu0 5;
    let dsu1 %DisjointSet unwrap_ok new 3;
    let r1 %Result bool Diag same &dsu1 0 4;
    let ok0 %bool is_err r0;
    let ok1 %bool is_err r1;
    free dsu0
    free dsu1
    let dsu2 %DisjointSet unwrap_ok new 3;
    let recovered_len %i32 match union dsu2 0 5:
        Result::Ok next:
            free next
            sub 0 1
        Result::Err e:
            let recovered %DisjointSet disjoint_set_update_error_owner e
            let actual_len %i32 len &recovered
            free recovered
            actual_len
    let report:
        test_report_new "disjoint_set_invalid_index"
        |> test_report_push assert "find invalid index errs" ok0
        |> test_report_push assert "same invalid index errs" ok1
        |> test_report_push assert_eq_i32 "union error returns owner len" 3 recovered_len
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
