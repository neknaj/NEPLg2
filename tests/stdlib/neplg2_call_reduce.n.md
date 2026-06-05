# NEPLg2 self-host call reduction input boundary

## direct_call_and_fail_closed_errors

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "neplg2/core/check/expr" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1 checks_push checks0 check_eq_i32 0 selfhost_check_expr_stage0
    let shown checks_print_report checks1
    checks_exit_code shown
```

## expression_line_segment_connects_to_call_reduction

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "neplg2/core/check/expr" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1 checks_push checks0 check_eq_i32 0 selfhost_check_expr_stage1_body_line
    let shown checks_print_report checks1
    checks_exit_code shown
```

## literal_char_payload_decode

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/char" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/check/expr/argument_payload" as *
#import "neplg2/core/check/expr/literal_payload" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    unwrap v::get tokens idx

fn char_item_for %fn SelfhostToken fn i32 SelfhostExprPrefixItem \token\token_index:
    SelfhostExprPrefixItem SelfhostExprPrefixItemKind::CharLiteral token_index field::get token "span"

fn char_payload_code_is %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError fn i32 Result unit str \result\expected:
    match result:
        Result::Ok argument:
            match field::get argument "kind":
                SelfhostCheckedArgumentKind::CharLiteral value:
                    check_eq_i32 expected char_to_i32 value
                _:
                    Result::Err "checked argument was not CharLiteral"
        Result::Err _error:
            Result::Err "char literal payload decode failed"

fn char_payload_error_is_multiple_scalars %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::CharMultipleScalars:
                    Result::Ok unit
                _:
                    Result::Err "char literal error was not CharMultipleScalars"
        Result::Ok _argument:
            Result::Err "multi-scalar char literal unexpectedly decoded"

fn char_payload_error_is_escape_unsupported %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::CharEscapeUnsupported:
                    Result::Ok unit
                _:
                    Result::Err "char literal error was not CharEscapeUnsupported"
        Result::Ok _argument:
            Result::Err "unsupported char escape unexpectedly decoded"

fn char_payload_error_is_invalid_scalar %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::CharInvalidScalar:
                    Result::Ok unit
                _:
                    Result::Err "char literal error was not CharInvalidScalar"
        Result::Ok _argument:
            Result::Err "invalid char scalar unexpectedly decoded"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "'\\n' '\\u{3042}' 'ab' '\\q' '\\u{110000}'"
    match lex_all source:
        Result::Ok tokens:
            let char_ty %SelfhostTypeId selfhost_type_id_new 42
            let t0 %SelfhostToken token_at &tokens 0
            let t1 %SelfhostToken token_at &tokens 1
            let t2 %SelfhostToken token_at &tokens 2
            let t3 %SelfhostToken token_at &tokens 3
            let t4 %SelfhostToken token_at &tokens 4
            let item0 %SelfhostExprPrefixItem char_item_for t0 0
            let item1 %SelfhostExprPrefixItem char_item_for t1 1
            let item2 %SelfhostExprPrefixItem char_item_for t2 2
            let item3 %SelfhostExprPrefixItem char_item_for t3 3
            let item4 %SelfhostExprPrefixItem char_item_for t4 4
            let decoded0 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item0 0 1 char_ty
            let decoded1 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item1 1 2 char_ty
            let decoded2 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item2 2 3 char_ty
            let decoded3 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item3 3 4 char_ty
            let decoded4 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item4 4 5 char_ty
            let checks1:
                checks0
                |> checks_push char_payload_code_is decoded0 10
                |> checks_push char_payload_code_is decoded1 0x3042
                |> checks_push char_payload_error_is_multiple_scalars decoded2
                |> checks_push char_payload_error_is_escape_unsupported decoded3
                |> checks_push char_payload_error_is_invalid_scalar decoded4
            v::free tokens
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
