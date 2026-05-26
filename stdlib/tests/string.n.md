# stdlib/string.n.md

## string_len_and_concat

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_len_and_concat\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"concat length\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"from_i32 length\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let s:
        "hello"
        |> concat "world"
    let s1234 from_i32 1234;
    let report:
        test_report_new "string_len_and_concat"
        |> test_report_push assert_eq_i32 "concat length" 10 len s
        |> test_report_push assert_eq_i32 "from_i32 length" 4 len s1234
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_trim_and_slice

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_trim_and_slice\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"trimmed length\" expected=\"15\" actual=\"15\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"trimmed prefix suffix\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=str_eq label=\"slice content\" expected=\"main\" actual=\"main\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let src "  fn main(a: i32)  ";
    let trimmed str_trim src;
    let part str_slice trimmed 3 7;
    let report:
        test_report_new "string_trim_and_slice"
        |> test_report_push assert_eq_i32 "trimmed length" 15 len trimmed
        |> test_report_push assert "trimmed prefix suffix" and str_starts_with trimmed "fn" str_ends_with trimmed ")"
        |> test_report_push assert_str_eq "slice content" "main" part
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_split_and_builder

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_split_and_builder\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"second split part is b\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"builder text\" expected=\"Error: 404 Not Found\" actual=\"Error: 404 Not Found\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let first %StrSplitStep str_split_next "a--b--c" "--" 0
    let second %StrSplitStep str_split_next "a--b--c" "--" get first "next"
    let msg %str:
        string_builder_new
        |> sb_append "Error: "
        |> sb_append_i32 404
        |> sb_append " Not Found"
        |> sb_build
    let second_is_b %bool match get second "kind":
        StrSplitStepKind::Done:
            false
        StrSplitStepKind::Part:
            let second_start %i32 get second "start"
            let second_end %i32 get second "end"
            str_range_eq "a--b--c" second_start second_end "b"
    let report:
        test_report_new "string_split_and_builder"
        |> test_report_push assert "second split part is b" second_is_b
        |> test_report_push assert_str_eq "builder text" "Error: 404 Not Found" msg
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_byte_at

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_byte_at\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"byte 0\" expected=\"65\" actual=\"65\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"byte 1\" expected=\"90\" actual=\"90\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"byte 2 none\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/option" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let byte0 %Option i32 byte_at "AZ" 0
    let byte1 %Option i32 byte_at "AZ" 1
    let byte2 %Option i32 byte_at "AZ" 2
    let byte0_value %i32 unwrap_or byte0 -1
    let byte1_value %i32 unwrap_or byte1 -1
    let byte2_none %bool is_none byte2
    let report:
        test_report_new "string_byte_at"
        |> test_report_push assert_eq_i32 "byte 0" 65 byte0_value
        |> test_report_push assert_eq_i32 "byte 1" 90 byte1_value
        |> test_report_push assert "byte 2 none" byte2_none
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_find_byte_index

`find` が最初に一致した byte index を返し、空 pattern、未検出、source より長い pattern を区別できることを確認します。
NEPLg3 self-host lexer が改行区切りや delimiter を探す用途を想定し、改行を含む探索も固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_find_byte_index\" count=7 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"empty pattern\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"prefix\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"middle\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"suffix\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"delimiter\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"missing pattern\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"longer pattern\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/option" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let empty_result %Option i32 find "abc" ""
    let prefix_result %Option i32 find "abc" "ab"
    let middle_result %Option i32 find "abcabc" "ca"
    let suffix_result %Option i32 find "abc" "bc"
    let delimiter_result %Option i32 find "#target std\n#entry main" "\n#entry"
    let missing_result %Option i32 find "abc" "z"
    let longer_result %Option i32 find "ab" "abc"
    let empty_value %i32 unwrap_or empty_result -1
    let prefix_value %i32 unwrap_or prefix_result -1
    let middle_value %i32 unwrap_or middle_result -1
    let suffix_value %i32 unwrap_or suffix_result -1
    let delimiter_value %i32 unwrap_or delimiter_result -1
    let missing_none %bool is_none missing_result
    let longer_none %bool is_none longer_result
    let report:
        test_report_new "string_find_byte_index"
        |> test_report_push assert_eq_i32 "empty pattern" 0 empty_value
        |> test_report_push assert_eq_i32 "prefix" 0 prefix_value
        |> test_report_push assert_eq_i32 "middle" 2 middle_value
        |> test_report_push assert_eq_i32 "suffix" 1 suffix_value
        |> test_report_push assert_eq_i32 "delimiter" 11 delimiter_value
        |> test_report_push assert "missing pattern" missing_none
        |> test_report_push assert "longer pattern" longer_none
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_result_allocation_apis

`concat_result` / `str_slice_result` / `str_split_next` / `StringBuilder` の Result API が、所有権を明示した形で期待内容を返せることを確認します。
allocator failure を trap へ寄せず、owned `Vec str` split に戻らない self-host 用入口を固定するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_result_allocation_apis\" count=4 failed=0\nassertion index=0 status=ok kind=str_eq label=\"concat result\" expected=\"abcd\" actual=\"abcd\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"slice result\" expected=\"cde\" actual=\"cde\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"split middle\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=str_eq label=\"builder result\" expected=\"Error: 404 Not Found\" actual=\"Error: 404 Not Found\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *

fn assert_str_result_ok %fn str fn Result str str fn str TestAssertion \label\got\expected:
    match got:
        Result::Ok actual:
            assert_str_eq label expected actual
        Result::Err e:
            test_assertion_failed AssertionKind::StrEq label expected e concat label e

fn split_middle_is_b %fn str fn StrSplitStep bool \source\step:
    match get step "kind":
        StrSplitStepKind::Done:
            false
        StrSplitStepKind::Part:
            let mid_start %i32 get step "start"
            let mid_end %i32 get step "end"
            str_range_eq source mid_start mid_end "b"

fn main %impure fn unit i32 \unit:
    let builder_result %Result str str:
        match string_builder_new_result:
            Result::Err e:
                Result::Err e
            Result::Ok sb0:
                match sb_append_result sb0 "Error: ":
                    Result::Err e:
                        Result::Err e
                    Result::Ok sb1:
                        match sb_append_i32_result sb1 404:
                            Result::Err e:
                                Result::Err e
                            Result::Ok sb2:
                                match sb_append_result sb2 " Not Found":
                                    Result::Err e:
                                        Result::Err e
                                    Result::Ok sb3:
                                        sb_build_result sb3
    let report:
        test_report_new "string_result_allocation_apis"
        |> test_report_push assert_str_result_ok "concat result" concat_result "ab" "cd" "abcd"
        |> test_report_push assert_str_result_ok "slice result" str_slice_result "abcdef" 2 5 "cde"
        |> test_report_push assert "split middle" split_middle_is_b "a--b--c" str_split_next "a--b--c" "--" 3
        |> test_report_push assert_str_result_ok "builder result" builder_result "Error: 404 Not Found"
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_utf8_mem_result

`string_from_utf8_mem_result` が有効な UTF-8 を `str` へ複製し、invalid leading byte を拒否することを確認します。
`alloc/string` 自体の境界で UTF-8 不変条件を固定するための回帰テストです。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_utf8_mem_result\" count=3 failed=0\nassertion index=0 status=ok kind=bool label=\"copy valid utf8\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"invalid leading byte rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"invalid buffer dealloc\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut report test_report_new "string_utf8_mem_result";
    let src %str "こんにちは";
    match string_from_utf8_mem_result string_data_ptr src len src:
        Result::Ok copied:
            set report test_report_push report assert "copy valid utf8" test_str_eq src copied
        Result::Err e:
            set report test_report_push report test_assertion_fail "copy valid utf8" e;
    match alloc_region_bytes<u8> 1:
        Result::Err e:
            set report test_report_push report test_assertion_fail "invalid buffer alloc" e
        Result::Ok region:
            let data %MemPtr u8 region_ptr &region
            match store_u8 data 128:
                Result::Err e:
                    set report test_report_push report test_assertion_fail "invalid leading byte store" e
                Result::Ok _:
                    match string_from_utf8_mem_result data 1:
                        Result::Ok text:
                            set report test_report_push report test_assertion_failed AssertionKind::Bool "invalid leading byte rejected" "true" "false" text
                        Result::Err _e:
                            set report test_report_push report assert "invalid leading byte rejected" true;
            match dealloc_region<u8> region:
                Result::Ok _:
                    set report test_report_push report assert "invalid buffer dealloc" true
                Result::Err e:
                    set report test_report_push report test_assertion_fail "invalid buffer dealloc" e;
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_to_f64_parser

`to_f64` が整数部だけ、符号付き小数、先頭が小数点の値、指数表記を clean end-of-input で正しく受理し、
不正な末尾や digit 不足を `Result::Err` として返すことを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_to_f64_parser\" count=11 failed=0\nassertion index=0 status=ok kind=bool label=\"integer\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"signed fraction\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"leading fraction\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"integer exponent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"fraction exponent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"empty\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"sign only\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"dot only\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"missing exponent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"trailing byte\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"trailing fraction byte\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/cast" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn assert_f64_ok %fn str fn Result f64 i32 fn f64 TestAssertion \label\got\expected:
    match got:
        Result::Ok actual:
            assert label eq actual expected
        Result::Err _e:
            test_assertion_fail label "rejected"

fn assert_f64_err %fn str fn Result f64 i32 TestAssertion \label\got:
    match got:
        Result::Ok _actual:
            test_assertion_fail label "accepted"
        Result::Err _e:
            assert label true

fn main %impure fn unit i32 \unit:
    let expected_integer %f64 %f64 cast 123;
    let expected_signed_fraction %f64 div %f64 cast -3 %f64 cast 2;
    let expected_leading_fraction %f64 div %f64 cast 1 %f64 cast 2;
    let expected_integer_exp %f64 %f64 cast 100;
    let expected_fraction_exp %f64 %f64 cast 125;
    let report:
        test_report_new "string_to_f64_parser"
        |> test_report_push assert_f64_ok "integer" (to_f64 "123") expected_integer
        |> test_report_push assert_f64_ok "signed fraction" (to_f64 "-1.5") expected_signed_fraction
        |> test_report_push assert_f64_ok "leading fraction" (to_f64 ".5") expected_leading_fraction
        |> test_report_push assert_f64_ok "integer exponent" (to_f64 "1e2") expected_integer_exp
        |> test_report_push assert_f64_ok "fraction exponent" (to_f64 "1.25e2") expected_fraction_exp
        |> test_report_push assert_f64_err "empty" (to_f64 "")
        |> test_report_push assert_f64_err "sign only" (to_f64 "-")
        |> test_report_push assert_f64_err "dot only" (to_f64 ".")
        |> test_report_push assert_f64_err "missing exponent" (to_f64 "1e")
        |> test_report_push assert_f64_err "trailing byte" (to_f64 "1x")
        |> test_report_push assert_f64_err "trailing fraction byte" (to_f64 "1.2x")
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## string_slice_utf8_boundary

`str_slice_result` が UTF-8 の文字境界に揃った範囲だけを `str` として返し、
multi-byte 文字の途中で切る範囲を `Result::Err` にすることを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"string_slice_utf8_boundary\" count=5 failed=0\nassertion index=0 status=ok kind=bool label=\"full character\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"second character\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"cut end rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"cut start rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=str_eq label=\"unchecked invalid slice fallback\" expected=\"\" actual=\"\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn assert_slice_ok %fn str fn Result str str fn str TestAssertion \label\got\expected:
    match got:
        Result::Ok actual:
            assert label test_str_eq expected actual
        Result::Err e:
            test_assertion_failed AssertionKind::Bool label "true" "false" concat label e

fn assert_slice_err %fn str fn Result str str TestAssertion \label\got:
    match got:
        Result::Ok actual:
            test_assertion_failed AssertionKind::Bool label "true" "false" actual
        Result::Err _e:
            assert label true

fn main %impure fn unit i32 \unit:
    let report:
        test_report_new "string_slice_utf8_boundary"
        |> test_report_push assert_slice_ok "full character" (str_slice_result "あ" 0 3) "あ"
        |> test_report_push assert_slice_ok "second character" (str_slice_result "あい" 3 6) "い"
        |> test_report_push assert_slice_err "cut end rejected" (str_slice_result "あ" 0 1)
        |> test_report_push assert_slice_err "cut start rejected" (str_slice_result "あ" 1 3)
        |> test_report_push assert_str_eq "unchecked invalid slice fallback" "" str_slice "あ" 0 1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
