# tests/vec_collections.n.md

## vec_free_zero_and_grow_reallocates

[目的/もくてき]:
- `Vec` が `with_capacity 0` を typed empty storage として[扱/あつか]い、`free` とその[後/あと]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- typed empty storage
- `push`
- grow
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_free_zero_and_grow_reallocates\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"empty vec is empty\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"empty vec cap\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"grown vec len\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"reallocated vec stores item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let empty %Vec i32 unwrap_ok with_capacity 0;
    let empty_is_empty %bool is_empty &empty;
    let empty_cap %i32 cap &empty;
    free empty;
    let mut grown %Vec i32 unwrap_ok new;
    set grown unwrap_ok push grown 0;
    set grown unwrap_ok push grown 1;
    set grown unwrap_ok push grown 2;
    set grown unwrap_ok push grown 3;
    set grown unwrap_ok push grown 4;
    set grown unwrap_ok push grown 5;
    set grown unwrap_ok push grown 6;
    set grown unwrap_ok push grown 7;
    set grown unwrap_ok push grown 8;
    set grown unwrap_ok push grown 9;
    let grown_len %i32 len &grown;
    free grown;
    let mut next %Vec i32 unwrap_ok new;
    set next unwrap_ok push next 42;
    let top_ok %bool match get &next 0:
        Option::Some v:
            eq v 42
        Option::None:
            false
    free next;
    let report:
        test_report_new "vec_free_zero_and_grow_reallocates"
        |> test_report_push assert "empty vec is empty" empty_is_empty
        |> test_report_push assert_eq_i32 "empty vec cap" 0 empty_cap
        |> test_report_push assert_eq_i32 "grown vec len" 10 grown_len
        |> test_report_push assert "reallocated vec stores item" top_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_sort_merge_ret_releases_scratch_buffer

[目的/もくてき]:
- merge sort の[作業/さぎょう] buffer cleanup が trap せず、その[後/あと]の `Vec` [再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `sort_merge_ret`
- scratch buffer cleanup
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_sort_merge_ret_releases_scratch_buffer\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"first item sorted\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"last item sorted\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"scratch cleanup permits allocation\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let unsorted %Vec i32:
        unwrap_ok new
        |> push 5 |> uwok
        |> push 2 |> uwok
        |> push 4 |> uwok
        |> push 1 |> uwok
    let sorted %Vec i32 unwrap_ok sort_merge_ret<i32> unsorted;
    let first_ok %bool match get &sorted 0:
        Option::Some v:
            eq v 1
        Option::None:
            false
    let last_ok %bool match get &sorted 3:
        Option::Some v:
            eq v 5
        Option::None:
            false
    free sorted;
    let mut next %Vec i32 unwrap_ok new;
    set next unwrap_ok push next 7;
    let next_ok %bool match get &next 0:
        Option::Some v:
            eq v 7
        Option::None:
            false
    free next;
    let report:
        test_report_new "vec_sort_merge_ret_releases_scratch_buffer"
        |> test_report_push assert "first item sorted" first_ok
        |> test_report_push assert "last item sorted" last_ok
        |> test_report_push assert "scratch cleanup permits allocation" next_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## vec_negative_capacity_rejected

[目的/もくてき]:
- `with_capacity` が[負/ふ]の capacity を allocator に[渡/わた]さず、typed error として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- `StdErrorKind::InvalidOperation`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"vec_negative_capacity_rejected\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"negative capacity error\" expected=\"InvalidOperation\" actual=\"InvalidOperation\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let neg %i32 sub 0 1
    let actual %str match with_capacity<i32> neg:
        Result::Ok v:
            free v
            "Ok"
        Result::Err e:
            std_error_kind_str e
    let report:
        test_report_new "vec_negative_capacity_rejected"
        |> test_report_push assert_str_eq "negative capacity error" "InvalidOperation" actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
