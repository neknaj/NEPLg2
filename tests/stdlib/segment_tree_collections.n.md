# tests/segment_tree_collections.n.md

## segment_tree_pipe_usage

[目的/もくてき]:
- `SegmentTree` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `len`
- `replace`
- `add`
- `sum_range`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_pipe_usage\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"segment tree len\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"range sum\" expected=\"7\" actual=\"7\" message=\"\"\n"
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
    let st %SegmentTree:
        unwrap_ok new 5
        |> replace 0 2 |> uwok
        |> replace 2 4 |> uwok
        |> add 2 1 |> uwok
    let total %i32 unwrap_ok sum_range &st 0 3;
    let st_len %i32 len &st;
    free st
    let report:
        test_report_new "segment_tree_pipe_usage"
        |> test_report_push assert_eq_i32 "segment tree len" 5 st_len
        |> test_report_push assert_eq_i32 "range sum" 7 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## segment_tree_update_free_reallocates

[目的/もくてき]:
- `SegmentTree` が update 後に `free` しても trap せず、その後の[再確保/さいかくほ]と query が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `replace`
- `add`
- `sum_range`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_update_free_reallocates\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"sum after realloc\" expected=\"7\" actual=\"7\" message=\"\"\n"
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
    let st_free %SegmentTree:
        unwrap_ok new 5
        |> replace 0 2 |> uwok
        |> replace 2 4 |> uwok
        |> add 2 1 |> uwok
    free st_free
    let st_empty %SegmentTree unwrap_ok new 0;
    free st_empty
    let st0 %SegmentTree:
        unwrap_ok new 5
        |> replace 4 6 |> uwok
        |> add 4 1 |> uwok
    let total %i32 unwrap_ok sum_range &st0 4 5;
    free st0
    let report:
        test_report_new "segment_tree_update_free_reallocates"
        |> test_report_push assert_eq_i32 "sum after realloc" 7 total
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## segment_tree_update_error_returns_owner

[目的/もくてき]:
- `replace` / `add` の[範囲外/はんいがい] error が `SegmentTree` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `replace`
- `add`
- `segment_tree_update_error_owner`
- `len`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_update_error_returns_owner\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"replace error returns owner len\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"add error returns owner len\" expected=\"4\" actual=\"4\" message=\"\"\n"
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
    let st %SegmentTree unwrap_ok new 4;
    match replace st 8 1:
        Result::Ok next0:
            free next0
            0
        Result::Err e0:
            let st0 %SegmentTree segment_tree_update_error_owner e0
            let replace_len %i32 len &st0
            match add st0 9 3:
                Result::Ok next1:
                    free next1
                    0
                Result::Err e1:
                    let recovered %SegmentTree segment_tree_update_error_owner e1
                    let add_len %i32 len &recovered
                    free recovered
                    let report:
                        test_report_new "segment_tree_update_error_returns_owner"
                        |> test_report_push assert_eq_i32 "replace error returns owner len" 4 replace_len
                        |> test_report_push assert_eq_i32 "add error returns owner len" 4 add_len
                    let shown test_report_print_stdout report
                    test_report_exit_code shown
```

## segment_tree_negative_length_rejected

[目的/もくてき]:
- `new` が[負/ふ]の length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"segment_tree_negative_length_rejected\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"negative length error\" expected=\"CapacityExceeded\" actual=\"CapacityExceeded\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/segment_tree" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let neg %i32 sub 0 1
    match new neg:
        Result::Ok st:
            free st
            0
        Result::Err d:
            let name %str diag_std_error_kind_str d
            let report:
                test_report_new "segment_tree_negative_length_rejected"
                |> test_report_push assert_str_eq "negative length error" "CapacityExceeded" name
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
