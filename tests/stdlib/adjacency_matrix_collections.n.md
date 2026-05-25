## adjacency_matrix_pipe_usage

[目的/もくてき]:
- `AdjacencyMatrix` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `clear`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_pipe_usage\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"edge 1->3 present\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed edge absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"matrix len\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"clear removes edge\" expected=\"true\" actual=\"true\" message=\"\"\n"
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
        unwrap_ok new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> remove 3 5 |> uwok
    let ok0 %bool unwrap_ok contains &g0 1 3;
    let ok1 %bool not unwrap_ok contains &g0 3 5;
    let size %i32 len &g0;
    free g0
    let g2 %AdjacencyMatrix:
        unwrap_ok new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> clear
    let ok3 %bool not unwrap_ok contains &g2 5 1;
    free g2
    let report:
        test_report_new "adjacency_matrix_pipe_usage"
        |> test_report_push assert "edge 1->3 present" ok0
        |> test_report_push assert "removed edge absent" ok1
        |> test_report_push assert_eq_i32 "matrix len" 6 size
        |> test_report_push assert "clear removes edge" ok3
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## adjacency_matrix_free_releases_owned_storage

[目的/もくてき]:
- `AdjacencyMatrix.free` が owner [管理/かんり]している matrix storage を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `insert`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_free_releases_owned_storage\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free permits later allocation\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let g0 %AdjacencyMatrix:
        unwrap_ok new 6
        |> insert 1 3 |> uwok
    free g0
    let g1 %AdjacencyMatrix:
        unwrap_ok new 6
        |> insert 2 4 |> uwok
    free g1
    let report:
        test_report_new "adjacency_matrix_free_releases_owned_storage"
        |> test_report_push assert "free permits later allocation" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## adjacency_matrix_update_error_recovers_owner

[目的/もくてき]:
- `insert` / `remove` の[範囲外/はんいがい] error が `AdjacencyMatrix` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `insert`
- `remove`
- `adjacency_matrix_update_error_owner`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_update_error_recovers_owner\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"insert error recovers owner\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"remove error recovers owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
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
    let g0 %AdjacencyMatrix unwrap_ok new 6;
    let ok0 %bool:
        match insert g0 6 0:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %AdjacencyMatrix adjacency_matrix_update_error_owner e
                let ok %bool eq len &recovered 6
                free recovered
                ok
    let g1 %AdjacencyMatrix unwrap_ok new 6;
    let ok1 %bool:
        match remove g1 2 9:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %AdjacencyMatrix adjacency_matrix_update_error_owner e
                let ok %bool eq len &recovered 6
                free recovered
                ok
    let report:
        test_report_new "adjacency_matrix_update_error_recovers_owner"
        |> test_report_push assert "insert error recovers owner" ok0
        |> test_report_push assert "remove error recovers owner" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## adjacency_matrix_non_positive_length_rejected

[目的/もくてき]:
- `new` が 0 以下の vertex length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"adjacency_matrix_non_positive_length_rejected\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"non-positive length error\" expected=\"CapacityExceeded\" actual=\"CapacityExceeded\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    match new 0:
        Result::Ok g:
            free g
            0
        Result::Err d:
            let name %str diag_std_error_kind_str d
            let report:
                test_report_new "adjacency_matrix_non_positive_length_rejected"
                |> test_report_push assert_str_eq "non-positive length error" "CapacityExceeded" name
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
