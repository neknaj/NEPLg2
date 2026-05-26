# stdlib/error.n.md

`alloc/diag/error` の[値/あたい]モデルを[確/たし]かめるためのテストです。
ここでは[表示/ひょうじ]ではなく、`StdErrorKind` / `Diag` / `Diags` / `Outcome` の[構築/こうちく]と
[補助/ほじょ] API が[期待/きたい]どおりに[振/ふる]る[舞/ま]うかを[確認/かくにん]します。

## std_error_kind_and_diag_value_model

[目的/もくてき]:
- `StdErrorKind` と `Diag` の[基本/きほん] API が[値/あたい]として[正/ただ]しく[扱/あつか]えることを[確/たし]かめます。
- span / note / help / source が `Diag` に[付与/ふよ]でき、`Diags` に[集約/しゅうやく]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `std_error_kind_str`
- `diag_error`
- `diag_with_span`
- `diag_add_note`
- `diag_add_help`
- `diag_with_source`
- `diags_one`
- `diags_push`
- `diags_len`
- `diags_has_errors`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"std_error_kind_and_diag_value_model\" count=8 failed=0\nassertion index=0 status=ok kind=str_eq label=\"Failure kind string\" expected=\"Failure\" actual=\"Failure\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"OutOfMemory kind string\" expected=\"OutOfMemory\" actual=\"OutOfMemory\" message=\"\"\nassertion index=2 status=ok kind=str_eq label=\"diag message\" expected=\"with source\" actual=\"with source\" message=\"\"\nassertion index=3 status=ok kind=str_eq label=\"diag kind string\" expected=\"Failure\" actual=\"Failure\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"span file id\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=5 status=ok kind=str_eq label=\"source text\" expected=\"parser\" actual=\"parser\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"diags length\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"diags has errors\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut report test_report_new "std_error_kind_and_diag_value_model";
    set report test_report_push report assert_str_eq "Failure kind string" "Failure" std_error_kind_str StdErrorKind::Failure;
    set report test_report_push report assert_str_eq "OutOfMemory kind string" "OutOfMemory" std_error_kind_str StdErrorKind::OutOfMemory;

    let sp %Span Span 4 5 6;
    let d0 %Diag diag_error StdErrorKind::Failure "with source";
    let d1 %Diag diag_with_span d0 sp;
    let d2 %Diag diag_add_note d1 "check input";
    let d3 %Diag diag_add_help d2 "doc: std/test";
    let d4 %Diag diag_with_source d3 "parser";

    set report test_report_push report assert_str_eq "diag message" "with source" *field::get_ref &d4 "message";
    set report test_report_push report assert_str_eq "diag kind string" "Failure" diag_std_error_kind_str &d4;

    match *field::get_ref &d4 "span":
        Option::Some got:
            set report test_report_push report assert_eq_i32 "span file id" 4 field::get got "file_id";
        Option::None:
            set report test_report_push report test_assertion_failed AssertionKind::EqI32 "span file id" "4" "None" "expected span";

    match *field::get_ref &d4 "source":
        Option::Some src:
            set report test_report_push report assert_str_eq "source text" "parser" src;
        Option::None:
            set report test_report_push report test_assertion_failed AssertionKind::StrEq "source text" "parser" "None" "expected source";

    let ds0 %Diags diags_one d4;
    let ds1 %Diags diags_push ds0 diag_warn "careful";
    set report test_report_push report assert_eq_i32 "diags length" 2 diags_len &ds1;
    set report test_report_push report assert "diags has errors" diags_has_errors ds1;
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## outcome_helpers_keep_result_and_diags_separate

[目的/もくてき]:
- `Outcome` が `result` と `diags` を[別軸/べつじく]で[保持/ほじ]することを[確/たし]かめます。
- `Result` をそのまま `Outcome` に[昇格/しょうかく]できる helper の[使/つか]い[方/かた]を[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `outcome_ok`
- `outcome_err`
- `outcome_with_diags`
- `result_to_outcome`
- `outcome_result`
- `outcome_is_ok`
- `outcome_is_err`
- `outcome_diags_or_empty`
- `outcome_has_errors`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"outcome_helpers_keep_result_and_diags_separate\" count=14 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"ok0 result\" expected=\"42\" actual=\"42\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"ok0 empty diags\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"ok1 result\" expected=\"42\" actual=\"42\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"ok1 is ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"ok1 is not err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"ok1 diags length\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"ok2 warns only\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"replace result\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=8 status=ok kind=str_eq label=\"err0 kind\" expected=\"IoError\" actual=\"IoError\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"err0 is not ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"err0 is err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=eq_i32 label=\"err0 empty diags\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"err0 has no errors\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=str_eq label=\"err1 kind\" expected=\"ParseError\" actual=\"ParseError\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn assert_result_ok_i32 %fn str fn Result i32 StdErrorKind fn i32 TestAssertion \label\got\expected:
    match got:
        Result::Ok v:
            assert_eq_i32 label expected v
        Result::Err kind:
            test_assertion_fail label std_error_kind_str kind

fn assert_kind_io_error %fn str fn Result i32 StdErrorKind TestAssertion \label\got:
    match got:
        Result::Ok _v:
            test_assertion_failed AssertionKind::StrEq label "IoError" "Ok" "expected Err IoError"
        Result::Err kind:
            match kind:
                StdErrorKind::IoError:
                    test_assertion_passed AssertionKind::StrEq label "IoError" "IoError"
                StdErrorKind::Failure:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "Failure" "expected IoError"
                StdErrorKind::OutOfMemory:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "OutOfMemory" "expected IoError"
                StdErrorKind::EmptyCollection:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "EmptyCollection" "expected IoError"
                StdErrorKind::IndexOutOfBounds:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "IndexOutOfBounds" "expected IoError"
                StdErrorKind::KeyNotFound:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "KeyNotFound" "expected IoError"
                StdErrorKind::CapacityExceeded:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "CapacityExceeded" "expected IoError"
                StdErrorKind::InvalidOperation:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "InvalidOperation" "expected IoError"
                StdErrorKind::InvalidUtf8:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "InvalidUtf8" "expected IoError"
                StdErrorKind::ParseError:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "ParseError" "expected IoError"
                StdErrorKind::Other:
                    test_assertion_failed AssertionKind::StrEq label "IoError" "Other" "expected IoError"

fn assert_kind_parse_error %fn str fn Result i32 StdErrorKind TestAssertion \label\got:
    match got:
        Result::Ok _v:
            test_assertion_failed AssertionKind::StrEq label "ParseError" "Ok" "expected Err ParseError"
        Result::Err kind:
            match kind:
                StdErrorKind::ParseError:
                    test_assertion_passed AssertionKind::StrEq label "ParseError" "ParseError"
                StdErrorKind::Failure:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "Failure" "expected ParseError"
                StdErrorKind::OutOfMemory:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "OutOfMemory" "expected ParseError"
                StdErrorKind::EmptyCollection:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "EmptyCollection" "expected ParseError"
                StdErrorKind::IndexOutOfBounds:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "IndexOutOfBounds" "expected ParseError"
                StdErrorKind::KeyNotFound:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "KeyNotFound" "expected ParseError"
                StdErrorKind::CapacityExceeded:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "CapacityExceeded" "expected ParseError"
                StdErrorKind::InvalidOperation:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "InvalidOperation" "expected ParseError"
                StdErrorKind::InvalidUtf8:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "InvalidUtf8" "expected ParseError"
                StdErrorKind::IoError:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "IoError" "expected ParseError"
                StdErrorKind::Other:
                    test_assertion_failed AssertionKind::StrEq label "ParseError" "Other" "expected ParseError"

fn main %impure fn unit i32 \unit:
    let mut report test_report_new "outcome_helpers_keep_result_and_diags_separate";
    let ok0 %Outcome i32 StdErrorKind outcome_ok 42;
    set report test_report_push report assert_result_ok_i32 "ok0 result" outcome_result &ok0 42;
    set report test_report_push report assert_eq_i32 "ok0 empty diags" 0 diags_len outcome_diags_or_empty ok0;

    let ds %Diags diags_one diag_warn "careful";
    let ok1_base %Outcome i32 StdErrorKind outcome_ok 42;
    let ok1 %Outcome i32 StdErrorKind outcome_with_diags ok1_base ds;
    set report test_report_push report assert_result_ok_i32 "ok1 result" outcome_result &ok1 42;
    set report test_report_push report assert "ok1 is ok" outcome_is_ok &ok1;
    set report test_report_push report assert "ok1 is not err" not outcome_is_err &ok1;
    set report test_report_push report assert_eq_i32 "ok1 diags length" 1 diags_len outcome_diags_or_empty ok1;
    let ok2_base %Outcome i32 StdErrorKind outcome_ok 42;
    let ok2 %Outcome i32 StdErrorKind outcome_with_diags ok2_base diags_one diag_warn "careful";
    set report test_report_push report assert "ok2 warns only" not outcome_has_errors ok2;

    let replace0_base %Outcome i32 StdErrorKind outcome_ok 7;
    let replace0 %Outcome i32 StdErrorKind outcome_with_diags replace0_base diags_one diag_warn "old";
    let replace1 %Outcome i32 StdErrorKind outcome_with_diags replace0 diags_one diag_warn "new";
    set report test_report_push report assert_result_ok_i32 "replace result" outcome_result replace1 7;

    let err0 %Outcome i32 StdErrorKind outcome_err StdErrorKind::IoError;
    set report test_report_push report assert_kind_io_error "err0 kind" outcome_result &err0;
    set report test_report_push report assert "err0 is not ok" not outcome_is_ok &err0;
    set report test_report_push report assert "err0 is err" outcome_is_err &err0;
    set report test_report_push report assert_eq_i32 "err0 empty diags" 0 diags_len outcome_diags_or_empty err0;
    let err0_empty %Outcome i32 StdErrorKind outcome_err StdErrorKind::IoError;
    set report test_report_push report assert "err0 has no errors" not outcome_has_errors err0_empty;

    let err1_result %Result i32 StdErrorKind Result::Err StdErrorKind::ParseError;
    let err1 %Outcome i32 StdErrorKind:
        result_to_outcome err1_result
    set report test_report_push report assert_kind_parse_error "err1 kind" outcome_result &err1;
    let shown test_report_print_stdout report
    test_report_exit_code shown
```


## result_and_outcome_common_helpers

[目的/もくてき]:
- `Result` と `Outcome` を[同/おな]じ helper [名/めい]で[扱/あつか]えることを[確/たし]かめます。
- [軽量/けいりょう]な API は `Result` のまま、rich な API は `Outcome` で[返/かえ]しても、[呼/よ]び[出/だ]し[側/がわ]の[読/よ]み[取/と]り helper を[共通化/きょうつうか]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `into_outcome`
- `result_like_result`
- `result_like_is_ok`
- `result_like_is_err`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"result_and_outcome_common_helpers\" count=8 failed=0\nassertion index=0 status=ok kind=bool label=\"r0 is ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"o0 is ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"r0 is not err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"o0 is not err\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"r0 result\" expected=\"9\" actual=\"9\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"o0 result\" expected=\"9\" actual=\"9\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"o2 is ok\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"o2 diags length\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let mut report test_report_new "result_and_outcome_common_helpers";
    let r0 %Result i32 StdErrorKind Result::Ok 9;
    let o0 %Outcome i32 StdErrorKind into_outcome r0;
    set report test_report_push report assert "r0 is ok" result_like_is_ok r0;
    set report test_report_push report assert "o0 is ok" result_like_is_ok &o0;
    set report test_report_push report assert "r0 is not err" not result_like_is_err r0;
    set report test_report_push report assert "o0 is not err" not result_like_is_err &o0;

    match result_like_result r0:
        Result::Ok v:
            set report test_report_push report assert_eq_i32 "r0 result" 9 v;
        Result::Err _e:
            set report test_report_push report test_assertion_fail "r0 result" "expected result ok";

    match result_like_result &o0:
        Result::Ok v:
            set report test_report_push report assert_eq_i32 "o0 result" 9 v;
        Result::Err _e:
            set report test_report_push report test_assertion_fail "o0 result" "expected outcome ok";

    let ds %Diags diags_one diag_warn "careful";
    let o1_base %Outcome i32 StdErrorKind outcome_ok 3;
    let o1 %Outcome i32 StdErrorKind outcome_with_diags o1_base ds;
    let o2 %Outcome i32 StdErrorKind into_outcome o1;
    set report test_report_push report assert "o2 is ok" result_like_is_ok &o2;
    set report test_report_push report assert_eq_i32 "o2 diags length" 1 diags_len outcome_diags_or_empty o2;
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
