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

neplg2:test
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

fn main <()*>i32> ():
    let mut checks checks_new;
    set checks checks_push checks check_str_eq "Failure" std_error_kind_str StdErrorKind::Failure;
    set checks checks_push checks check_str_eq "OutOfMemory" std_error_kind_str StdErrorKind::OutOfMemory;

    let sp <Span> Span 4 5 6;
    let d0 <Diag> diag_error StdErrorKind::Failure "with source";
    let d1 <Diag> diag_with_span d0 sp;
    let d2 <Diag> diag_add_note d1 "check input";
    let d3 <Diag> diag_add_help d2 "doc: std/test";
    let d4 <Diag> diag_with_source d3 "parser";

    set checks checks_push checks check_str_eq "with source" *field::get_ref &d4 "message";
    set checks checks_push checks check_str_eq "Failure" diag_std_error_kind_str &d4;

    match *field::get_ref &d4 "span":
        Option::Some got:
            set checks checks_push checks check_eq_i32 4 get got "file_id";
        Option::None:
            set checks checks_push checks Result<(),str>::Err "expected span";

    match *field::get_ref &d4 "source":
        Option::Some src:
            set checks checks_push checks check_str_eq "parser" src;
        Option::None:
            set checks checks_push checks Result<(),str>::Err "expected source";

    let ds0 <Diags> diags_one d4;
    let ds1 <Diags> diags_push ds0 diag_warn "careful";
    set checks checks_push checks check_eq_i32 2 diags_len &ds1;
    set checks checks_push checks check diags_has_errors &ds1;
    let shown checks_print_report checks;
    checks_exit_code shown
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

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    let ok0 <Outcome<i32, StdErrorKind>> outcome_ok<i32, StdErrorKind> 42;
    match outcome_result &ok0:
        Result::Ok v:
            set checks checks_push checks check_eq_i32 42 v;
        Result::Err kind:
            match kind:
                StdErrorKind::Failure:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::OutOfMemory:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::EmptyCollection:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::IndexOutOfBounds:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::KeyNotFound:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::CapacityExceeded:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::InvalidOperation:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::InvalidUtf8:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::ParseError:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::IoError:
                    set checks checks_push checks Result<(),str>::Err "expected ok";
                StdErrorKind::Other:
                    set checks checks_push checks Result<(),str>::Err "expected ok";

    set checks checks_push checks check_eq_i32 0 diags_len outcome_diags_or_empty ok0;

    let ds <Diags> diags_one diag_warn "careful";
    let ok1 <Outcome<i32, StdErrorKind>> outcome_with_diags outcome_ok<i32, StdErrorKind> 42 ds;
    match outcome_result &ok1:
        Result::Ok v:
            set checks checks_push checks check_eq_i32 42 v;
        Result::Err _kind:
            set checks checks_push checks Result<(),str>::Err "expected ok result";
    set checks checks_push checks check outcome_is_ok &ok1;
    set checks checks_push checks check not outcome_is_err &ok1;
    set checks checks_push checks check_eq_i32 1 diags_len outcome_diags_or_empty ok1;
    let ok2 <Outcome<i32, StdErrorKind>> outcome_with_diags outcome_ok<i32, StdErrorKind> 42 diags_one diag_warn "careful";
    set checks checks_push checks check not outcome_has_errors ok2;

    let err0 <Outcome<i32, StdErrorKind>> outcome_err<i32, StdErrorKind> StdErrorKind::IoError;
    match outcome_result &err0:
        Result::Ok _v:
            set checks checks_push checks Result<(),str>::Err "expected err";
        Result::Err kind:
            match kind:
                StdErrorKind::IoError:
                    set checks checks_push checks Result<(),str>::Ok ();
                StdErrorKind::Failure:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::OutOfMemory:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::EmptyCollection:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::IndexOutOfBounds:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::KeyNotFound:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::CapacityExceeded:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::InvalidOperation:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::InvalidUtf8:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::ParseError:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
                StdErrorKind::Other:
                    set checks checks_push checks Result<(),str>::Err "expected IoError";
    set checks checks_push checks check not outcome_is_ok &err0;
    set checks checks_push checks check outcome_is_err &err0;
    set checks checks_push checks check_eq_i32 0 diags_len outcome_diags_or_empty err0;
    set checks checks_push checks check not outcome_has_errors outcome_err<i32, StdErrorKind> StdErrorKind::IoError;

    let err1 <Outcome<i32, StdErrorKind>>:
        result_to_outcome<i32, StdErrorKind> Result::Err StdErrorKind::ParseError
    match get err1 "result":
        Result::Ok _v:
            set checks checks_push checks Result<(),str>::Err "expected err";
        Result::Err kind:
            match kind:
                StdErrorKind::ParseError:
                    set checks checks_push checks Result<(),str>::Ok ();
                StdErrorKind::Failure:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::OutOfMemory:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::EmptyCollection:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::IndexOutOfBounds:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::KeyNotFound:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::CapacityExceeded:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::InvalidOperation:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::InvalidUtf8:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::IoError:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
                StdErrorKind::Other:
                    set checks checks_push checks Result<(),str>::Err "expected ParseError";
    let shown checks_print_report checks;
    checks_exit_code shown
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

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    let r0 <Result<i32, StdErrorKind>> Result::Ok 9;
    let o0 <Outcome<i32, StdErrorKind>> into_outcome r0;
    set checks checks_push checks check result_like_is_ok r0;
    set checks checks_push checks check result_like_is_ok &o0;
    set checks checks_push checks check not result_like_is_err r0;
    set checks checks_push checks check not result_like_is_err &o0;

    match result_like_result r0:
        Result::Ok v:
            set checks checks_push checks check_eq_i32 9 v;
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "expected result ok";

    match result_like_result &o0:
        Result::Ok v:
            set checks checks_push checks check_eq_i32 9 v;
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "expected outcome ok";

    let ds <Diags> diags_one diag_warn "careful";
    let o1 <Outcome<i32, StdErrorKind>> outcome_with_diags outcome_ok<i32, StdErrorKind> 3 ds;
    let o2 <Outcome<i32, StdErrorKind>> into_outcome o1;
    set checks checks_push checks check result_like_is_ok &o2;
    set checks checks_push checks check_eq_i32 1 diags_len outcome_diags_or_empty o2;
    let shown checks_print_report checks;
    checks_exit_code shown
```
