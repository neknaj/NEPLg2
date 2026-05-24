# tests/fenwick_collections.n.md

## fenwick_pipe_usage

[目的/もくてき]:
- `Fenwick` が bare API と `Result` を[組/く]み[合/あ]わせた pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `add`
- `len`
- `sum_prefix`
- `sum_range`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_pipe_usage\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"fenwick len\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"prefix sum 0..5\" expected=\"14\" actual=\"14\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"range sum 2..5\" expected=\"12\" actual=\"12\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let fw %Fenwick:
        unwrap_ok<Fenwick, Diag> fw::new 6
        |> fw::add 0 2 |> uwok
        |> fw::add 2 5 |> uwok
        |> fw::add 4 7 |> uwok
    let size %i32 fw::len &fw;
    let prefix5 %i32 unwrap_ok<i32, Diag> fw::sum_prefix &fw 5;
    let range_2_5 %i32 unwrap_ok<i32, Diag> fw::sum_range &fw 2 5;
    fw::free fw
    let report:
        test_report_new "fenwick_pipe_usage"
        |> test_report_push assert_eq_i32 "fenwick len" 6 size
        |> test_report_push assert_eq_i32 "prefix sum 0..5" 14 prefix5
        |> test_report_push assert_eq_i32 "range sum 2..5" 12 range_2_5
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## fenwick_free_releases_owned_storage

[目的/もくてき]:
- `Fenwick.free` が owner [管理/かんり]している 1-indexed `bit` [配列/はいれつ]を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `add`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_free_releases_owned_storage\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free permits later allocation\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let fw0 %Fenwick:
        unwrap_ok<Fenwick, Diag> fw::new 6
        |> fw::add 1 3 |> uwok
    fw::free fw0
    let fw1 %Fenwick:
        unwrap_ok<Fenwick, Diag> fw::new 6
        |> fw::add 2 5 |> uwok
    fw::free fw1
    let report:
        test_report_new "fenwick_free_releases_owned_storage"
        |> test_report_push assert "free permits later allocation" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## fenwick_add_error_returns_owner

[目的/もくてき]:
- `add` の[範囲外/はんいがい] error が `Fenwick` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `add`
- `add_error_tree`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_add_error_returns_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"add error recovers owner len\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let fw %Fenwick unwrap_ok<Fenwick, Diag> fw::new 4;
    match fw::add fw 8 3:
        Result::Ok next:
            fw::free next
            0
        Result::Err e:
            let recovered %Fenwick fw::add_error_tree e
            let size %i32 fw::len &recovered
            fw::free recovered
            let report:
                test_report_new "fenwick_add_error_returns_owner"
                |> test_report_push assert_eq_i32 "add error recovers owner len" 4 size
            let shown test_report_print_stdout report
            test_report_exit_code shown
```

## fenwick_negative_length_rejected

[目的/もくてき]:
- `new` が[負/ふ]の length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"fenwick_negative_length_rejected\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"negative length error\" expected=\"CapacityExceeded\" actual=\"CapacityExceeded\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let neg %i32 sub 0 1
    match fw::new neg:
        Result::Ok fw:
            fw::free fw
            0
        Result::Err d:
            let name %str diag_std_error_kind_str d
            let report:
                test_report_new "fenwick_negative_length_rejected"
                |> test_report_push assert_str_eq "negative length error" "CapacityExceeded" name
            let shown test_report_print_stdout report
            test_report_exit_code shown
```
