# NEPLg2 self-host source span

## checked_constructor_rejects_invalid_ranges

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "std/test" as *

fn expect_negative_file %fn Result SelfhostSourceSpan SelfhostSourceSpanBuildError Result unit str \r:
    match r:
        Result::Err e:
            match e:
                SelfhostSourceSpanBuildError::NegativeFileId:
                    Result::Ok unit
                SelfhostSourceSpanBuildError::NegativeStart:
                    Result::Err "expected NegativeFileId"
                SelfhostSourceSpanBuildError::EndBeforeStart:
                    Result::Err "expected NegativeFileId"
                SelfhostSourceSpanBuildError::DifferentFile:
                    Result::Err "expected NegativeFileId"
        Result::Ok _span:
            Result::Err "expected NegativeFileId"

fn expect_negative_start %fn Result SelfhostSourceSpan SelfhostSourceSpanBuildError Result unit str \r:
    match r:
        Result::Err e:
            match e:
                SelfhostSourceSpanBuildError::NegativeFileId:
                    Result::Err "expected NegativeStart"
                SelfhostSourceSpanBuildError::NegativeStart:
                    Result::Ok unit
                SelfhostSourceSpanBuildError::EndBeforeStart:
                    Result::Err "expected NegativeStart"
                SelfhostSourceSpanBuildError::DifferentFile:
                    Result::Err "expected NegativeStart"
        Result::Ok _span:
            Result::Err "expected NegativeStart"

fn expect_end_before_start %fn Result SelfhostSourceSpan SelfhostSourceSpanBuildError Result unit str \r:
    match r:
        Result::Err e:
            match e:
                SelfhostSourceSpanBuildError::NegativeFileId:
                    Result::Err "expected EndBeforeStart"
                SelfhostSourceSpanBuildError::NegativeStart:
                    Result::Err "expected EndBeforeStart"
                SelfhostSourceSpanBuildError::EndBeforeStart:
                    Result::Ok unit
                SelfhostSourceSpanBuildError::DifferentFile:
                    Result::Err "expected EndBeforeStart"
        Result::Ok _span:
            Result::Err "expected EndBeforeStart"

fn expect_different_file %fn Result SelfhostSourceSpan SelfhostSourceSpanBuildError Result unit str \r:
    match r:
        Result::Err e:
            match e:
                SelfhostSourceSpanBuildError::NegativeFileId:
                    Result::Err "expected DifferentFile"
                SelfhostSourceSpanBuildError::NegativeStart:
                    Result::Err "expected DifferentFile"
                SelfhostSourceSpanBuildError::EndBeforeStart:
                    Result::Err "expected DifferentFile"
                SelfhostSourceSpanBuildError::DifferentFile:
                    Result::Ok unit
        Result::Ok _span:
            Result::Err "expected DifferentFile"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1:
        match source_span_new_result 0 2 5:
            Result::Ok span:
                match source_span_len span:
                    Option::Some len:
                        checks0
                        |> checks_push check_eq_i32 3 len
                        |> checks_push check source_span_contains span 3
                    Option::None:
                        checks_push checks0 Result::Err "valid span length was missing"
            Result::Err _e:
                checks_push checks0 Result::Err "valid span was rejected"
    let checks2 checks_push checks1 expect_negative_file source_span_new_result -1 0 1
    let checks3 checks_push checks2 expect_negative_start source_span_new_result 0 -1 1
    let checks4 checks_push checks3 expect_end_before_start source_span_new_result 0 5 2
    let checks5:
        match source_span_empty_result 0 4:
            Result::Ok span:
                checks_push checks4 check source_span_is_valid span
            Result::Err _e:
                checks_push checks4 Result::Err "empty span was rejected"
    let invalid %SelfhostSourceSpan source_span_new_unchecked 0 5 2
    let checks6 checks_push checks5 check is_none source_span_len invalid
    let left %SelfhostSourceSpan source_span_new_unchecked 0 0 1
    let right %SelfhostSourceSpan source_span_new_unchecked 1 0 1
    let checks7 checks_push checks6 expect_different_file source_span_join_result left right
    let shown checks_print_report checks7
    checks_exit_code shown
```
