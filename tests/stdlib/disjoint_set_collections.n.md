# tests/disjoint_set_collections.n.md

## disjoint_set_pipe_usage

[目的/もくてき]:
- `DisjointSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `len`
- `union`
- `same`
- `size`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_pipe_usage\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"0 and 3 connected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"disjoint set len\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"component size\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let dsu0 %DisjointSet:
        unwrap_ok new 5
        |> union 0 1 |> uwok
        |> union 3 4 |> uwok
        |> union 1 4 |> uwok
    let ok0 %bool unwrap_ok same &dsu0 0 3;
    let dsu_len %i32 len &dsu0;
    free dsu0
    let dsu1 %DisjointSet:
        unwrap_ok new 5
        |> union 0 1 |> uwok
        |> union 3 4 |> uwok
        |> union 1 4 |> uwok
    let component_size %i32 unwrap_ok size &dsu1 4;
    free dsu1
    let report:
        test_report_new "disjoint_set_pipe_usage"
        |> test_report_push assert "0 and 3 connected" ok0
        |> test_report_push assert_eq_i32 "disjoint set len" 5 dsu_len
        |> test_report_push assert_eq_i32 "component size" 4 component_size
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## disjoint_set_union_free_reallocates

[目的/もくてき]:
- `DisjointSet` の union-by-size [更新/こうしん]と `free` が、内部の owned array cleanup で trap しないことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `union`
- `free`
- `same`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_union_free_reallocates\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"same after realloc\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let dsu_free %DisjointSet:
        unwrap_ok new 4
        |> union 0 1 |> uwok
        |> union 2 3 |> uwok
        |> union 1 2 |> uwok
    free dsu_free
    let dsu0 %DisjointSet:
        unwrap_ok new 4
        |> union 0 3 |> uwok
    let ok0 %bool unwrap_ok same &dsu0 0 3;
    free dsu0
    let report:
        test_report_new "disjoint_set_union_free_reallocates"
        |> test_report_push assert "same after realloc" ok0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## disjoint_set_new_zero_is_empty

[目的/もくてき]:
- `new 0` が[空/から]の union-find として[成功/せいこう]し、`free` と[後続/こうぞく]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new 0`
- `len`
- empty `find` の範囲外 error
- `free`
- [再確保/さいかくほ]

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_new_zero_is_empty\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"zero len\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"empty find rejects index\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"free empty succeeds\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"realloc root is zero\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let len_ok %bool match new 0:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok %bool eq len &dsu 0
            free dsu
            ok
    let find_err_ok %bool match new 0:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok %bool match find &dsu 0:
                Result::Ok _root:
                    false
                Result::Err _e:
                    true
            free dsu
            ok
    let free_ok %bool block:
        let empty %DisjointSet unwrap_ok new 0
        free empty
        true
    let realloc_ok %bool match new 1:
        Result::Err _e:
            false
        Result::Ok dsu:
            let ok %bool match find &dsu 0:
                Result::Ok root:
                    eq root 0
                Result::Err _e:
                    false
            free dsu
            ok
    let report:
        test_report_new "disjoint_set_new_zero_is_empty"
        |> test_report_push assert "zero len" len_ok
        |> test_report_push assert "empty find rejects index" find_err_ok
        |> test_report_push assert "free empty succeeds" free_ok
        |> test_report_push assert "realloc root is zero" realloc_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## disjoint_set_union_error_returns_owner

[目的/もくてき]:
- `union` の[範囲外/はんいがい] error が `DisjointSet` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `union`
- `disjoint_set_update_error_owner`
- `len`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_union_error_returns_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"union error returns owner len\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let dsu %DisjointSet unwrap_ok new 4;
    match union dsu 1 9:
        Result::Ok next:
            free next
            0
        Result::Err e:
            let recovered %DisjointSet disjoint_set_update_error_owner e
            let recovered_len %i32 len &recovered
            free recovered
            let report:
                test_report_new "disjoint_set_union_error_returns_owner"
                |> test_report_push assert_eq_i32 "union error returns owner len" 4 recovered_len
            let shown test_report_print_stdout report
            test_report_exit_code shown
```

## disjoint_set_negative_length_rejected

[目的/もくてき]:
- `new` が[負/ふ]の length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"disjoint_set_negative_length_rejected\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"negative length error\" expected=\"CapacityExceeded\" actual=\"CapacityExceeded\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/disjoint_set" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let neg %i32 sub 0 1
    match new neg:
        Result::Ok dsu:
            free dsu
            0
        Result::Err d:
            let name %str diag_std_error_kind_str d
            let report:
                test_report_new "disjoint_set_negative_length_rejected"
                |> test_report_push assert_str_eq "negative length error" "CapacityExceeded" name
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
