# NEPLg2 self-host diagnostic and outcome

## diagnostic_value_and_collection

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "std/test" as *
#import "core/field" as *

fn main <()*>i32> ():
    let mut checks checks_new
    let span <SelfhostSourceSpan> source_span_new 4 10 14
    let label <SelfhostDiagnosticLabel> selfhost_diag_label_new span "identifier"
    let diag0 <SelfhostDiagnostic> selfhost_diag_error SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::ImportAliasExpected "expected identifier"
    let diag1 <SelfhostDiagnostic> selfhost_diag_with_primary_label diag0 label
    let diag2 <SelfhostDiagnostic> selfhost_diag_with_note diag1 "while parsing import"
    set checks checks_push checks check_str_eq "error" selfhost_diag_severity_name field::get diag2 "severity"
    set checks checks_push checks check_str_eq "parser.import.alias_expected" selfhost_diag_code_name field::get diag2 "code"
    match field::get diag2 "primary_label":
        Option::Some got:
            let got_span <SelfhostSourceSpan> field::get got "span"
            set checks checks_push checks check_eq_i32 10 field::get got_span "start"
            set checks checks_push checks check_str_eq "identifier" field::get got "message"
        Option::None:
            set checks checks_push checks Result<(),str>::Err "expected primary label"
    match field::get diag2 "note":
        Option::Some note:
            set checks checks_push checks check_str_eq "while parsing import" note
        Option::None:
            set checks checks_push checks Result<(),str>::Err "expected note"

    match selfhost_diagnostics_one diag2:
        Result::Ok ds0:
            let warn <SelfhostDiagnostic> selfhost_diag_warning SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::RawBlockExpectedIndent "recovered"
            match selfhost_diagnostics_push ds0 warn:
                Result::Ok ds1:
                    set checks checks_push checks check_eq_i32 2 selfhost_diagnostics_len &ds1
                    set checks checks_push checks check selfhost_diagnostics_has_errors &ds1
                    match selfhost_diagnostics_get &ds1 1:
                        Option::Some got:
                            set checks checks_push checks check_str_eq "warning" selfhost_diag_severity_name field::get got "severity"
                        Option::None:
                            set checks checks_push checks Result<(),str>::Err "expected second diagnostic"
                    selfhost_diagnostics_free ds1
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "diagnostics push failed"
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "diagnostics one failed"

    let shown checks_print_report checks
    checks_exit_code shown
```

## outcome_keeps_result_and_diagnostics_separate

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/outcome" as *
#import "std/test" as *
#import "core/math" as *

fn main <()*>i32> ():
    let mut checks checks_new

    match selfhost_outcome_ok<i32,str> 42:
        Result::Ok ok0:
            set checks checks_push checks check_eq_i32 0 selfhost_outcome_diagnostics_len<i32,str> &ok0
            set checks checks_push checks check not selfhost_outcome_has_errors<i32,str> &ok0
            let warn <SelfhostDiagnostic> selfhost_diag_warning SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::RawBlockExpectedIndent "recovered"
            match selfhost_outcome_push_diagnostic<i32,str> ok0 warn @selfhost_outcome_ignore_i32 @selfhost_outcome_ignore_str:
                Result::Ok ok1:
                    set checks checks_push checks check_eq_i32 1 selfhost_outcome_diagnostics_len<i32,str> &ok1
                    set checks checks_push checks check not selfhost_outcome_has_errors<i32,str> &ok1
                    match selfhost_outcome_result<i32,str> ok1:
                        Result::Ok value:
                            set checks checks_push checks check_eq_i32 42 value
                        Result::Err _e:
                            set checks checks_push checks Result<(),str>::Err "expected ok result"
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "push warning failed"
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "outcome ok failed"

    match selfhost_outcome_err<i32,str> "bad":
        Result::Ok err0:
            let diag <SelfhostDiagnostic> selfhost_diag_error SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::TokenIndex "type mismatch"
            match selfhost_outcome_push_diagnostic<i32,str> err0 diag @selfhost_outcome_ignore_i32 @selfhost_outcome_ignore_str:
                Result::Ok err1:
                    set checks checks_push checks check_eq_i32 1 selfhost_outcome_diagnostics_len<i32,str> &err1
                    set checks checks_push checks check selfhost_outcome_has_errors<i32,str> &err1
                    match selfhost_outcome_result<i32,str> err1:
                        Result::Ok _value:
                            set checks checks_push checks Result<(),str>::Err "expected err result"
                        Result::Err msg:
                            set checks checks_push checks check_str_eq "bad" msg
                Result::Err _e:
                    set checks checks_push checks Result<(),str>::Err "push error diagnostic failed"
        Result::Err _e:
            set checks checks_push checks Result<(),str>::Err "outcome err failed"

    let shown checks_print_report checks
    checks_exit_code shown
```

## outcome_free_calls_result_payload_cleanup

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "okerr"
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/outcome" as *
#import "std/stdio" as *

fn print_payload <(str)*>()> (value):
    print value

fn main <()*>i32> ():
    match selfhost_outcome_ok<str,i32> "ok":
        Result::Ok ok_outcome:
            selfhost_outcome_free<str,i32> ok_outcome @print_payload @selfhost_outcome_ignore_i32
            match selfhost_outcome_err<i32,str> "err":
                Result::Ok err_outcome:
                    selfhost_outcome_free<i32,str> err_outcome @selfhost_outcome_ignore_i32 @print_payload
                    0
                Result::Err _e:
                    1
        Result::Err _e:
            1
```
