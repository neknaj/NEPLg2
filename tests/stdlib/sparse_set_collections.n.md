# tests/sparse_set_collections.n.md

## sparse_set_pipe_usage

[目的/もくてき]:
- `SparseSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `universe_len`
- `clear`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sparse_set_pipe_usage\" count=5 failed=0\nassertion index=0 status=ok kind=bool label=\"contains kept value\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed value absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"sparse set len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"universe len\" expected=\"12\" actual=\"12\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"clear len\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let s0 %SparseSet:
        unwrap_ok new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
    let ok0 %bool unwrap_ok contains &s0 9;
    let ok1 %bool not unwrap_ok contains &s0 5;
    let size %i32 len &s0;
    let universe %i32 universe_len &s0;
    let s1 %SparseSet clear s0;
    let cleared_size %i32 len &s1;
    free s1
    let report:
        test_report_new "sparse_set_pipe_usage"
        |> test_report_push assert "contains kept value" ok0
        |> test_report_push assert "removed value absent" ok1
        |> test_report_push assert_eq_i32 "sparse set len" 2 size
        |> test_report_push assert_eq_i32 "universe len" 12 universe
        |> test_report_push assert_eq_i32 "clear len" 0 cleared_size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sparse_set_clear_free_reallocates

[目的/もくてき]:
- `SparseSet` が insert/remove/clear 後に `free` しても trap せず、その後の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `clear`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sparse_set_clear_free_reallocates\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"contains after realloc\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let s_free %SparseSet:
        unwrap_ok new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
        |> clear
    free s_free
    let s0 %SparseSet:
        unwrap_ok new 12
        |> insert 7 |> uwok
    let ok0 %bool unwrap_ok contains &s0 7;
    free s0
    let report:
        test_report_new "sparse_set_clear_free_reallocates"
        |> test_report_push assert "contains after realloc" ok0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sparse_set_new_zero_is_empty

[目的/もくてき]:
- `new 0` が[空/から] domain の `SparseSet` として[成功/せいこう]し、`free` と[後続/こうぞく]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new 0`
- `universe_len`
- empty `contains` の範囲外 error
- `free`
- [再確保/さいかくほ]

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sparse_set_new_zero_is_empty\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"zero universe len\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty contains rejects index\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"free empty succeeds\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"realloc after empty free\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let len_ok %bool match new 0:
        Result::Err _e:
            false
        Result::Ok s:
            let ok %bool eq universe_len &s 0;
            free s
            ok
    let contains_err_ok %bool match new 0:
        Result::Err _e:
            false
        Result::Ok s:
            let r %Result bool Diag contains &s 0;
            free s
            match r:
                Result::Ok _v:
                    false
                Result::Err _e:
                    true
    let free_ok %bool block:
        let empty %SparseSet unwrap_ok new 0
        free empty
        true
    let realloc_ok %bool match new 2:
        Result::Err _e:
            false
        Result::Ok s0:
            match insert s0 1:
                Result::Err e:
                    let recovered %SparseSet sparse_set_update_error_owner e
                    free recovered
                    false
                Result::Ok s1:
                    let r %Result bool Diag contains &s1 1;
                    free s1
                    match r:
                        Result::Ok v:
                            v
                        Result::Err _e:
                            false
    let report:
        test_report_new "sparse_set_new_zero_is_empty"
        |> test_report_push assert "zero universe len" len_ok
        |> test_report_push assert "empty contains rejects index" contains_err_ok
        |> test_report_push assert "free empty succeeds" free_ok
        |> test_report_push assert "realloc after empty free" realloc_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
