# stdlib/adjacency_matrix.n.md

## adjacency_matrix_insert_remove_contains

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_insert_remove_contains\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"removed edge absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"kept edge present\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"matrix len\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let g %AdjacencyMatrix:
        unwrap_ok new 5
        |> insert 0 1 |> uwok
        |> insert 0 4 |> uwok
        |> insert 3 2 |> uwok
        |> remove 0 1 |> uwok
    let ok0 %bool not unwrap_ok contains &g 0 1;
    let ok1 %bool unwrap_ok contains &g 0 4;
    let size %i32 len &g;
    free g
    let report:
        test_report_new "adjacency_matrix_insert_remove_contains"
        |> test_report_push assert "removed edge absent" ok0
        |> test_report_push assert "kept edge present" ok1
        |> test_report_push assert_eq_i32 "matrix len" 5 size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## adjacency_matrix_clear

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_clear\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"clear removes edge\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let g0 %AdjacencyMatrix:
        unwrap_ok new 4
        |> insert 1 2 |> uwok
        |> clear
    let ok0 %bool not unwrap_ok contains &g0 1 2;
    free g0
    let report:
        test_report_new "adjacency_matrix_clear"
        |> test_report_push assert "clear removes edge" ok0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## adjacency_matrix_update_error_returns_owner

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_update_error_returns_owner\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"insert error returns owner\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"remove error returns owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let g0 %AdjacencyMatrix unwrap_ok new 5;
    let ok0 %bool:
        match insert g0 5 1:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %AdjacencyMatrix adjacency_matrix_update_error_owner e
                let ok %bool eq len &recovered 5
                free recovered
                ok
    let g1 %AdjacencyMatrix unwrap_ok new 5;
    let ok1 %bool:
        match remove g1 0 7:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %AdjacencyMatrix adjacency_matrix_update_error_owner e
                let ok %bool eq len &recovered 5
                free recovered
                ok
    let report:
        test_report_new "adjacency_matrix_update_error_returns_owner"
        |> test_report_push assert "insert error returns owner" ok0
        |> test_report_push assert "remove error returns owner" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
