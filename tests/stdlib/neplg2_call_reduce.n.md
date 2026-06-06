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

## literal_string_payload_decode

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
    ##: [8] ok
    ##: [9] ok
    ##: [10] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
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

fn string_token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    unwrap v::get tokens idx

fn string_item_for %fn SelfhostToken fn i32 SelfhostExprPrefixItem \token\token_index:
    SelfhostExprPrefixItem SelfhostExprPrefixItemKind::StringLiteral token_index field::get token "span"

fn string_payload_value_is %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError fn str Result unit str \result\expected:
    match result:
        Result::Ok argument:
            match field::get argument "kind":
                SelfhostCheckedArgumentKind::StrLiteral value:
                    check_str_eq expected value
                _:
                    Result::Err "checked argument was not StrLiteral"
        Result::Err _error:
            Result::Err "string literal payload decode failed"

fn string_payload_error_is_unsupported %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::StringEscapeUnsupported:
                    Result::Ok unit
                _:
                    Result::Err "string literal error was not StringEscapeUnsupported"
        Result::Ok _argument:
            Result::Err "unsupported string escape unexpectedly decoded"

fn string_payload_error_is_malformed %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::StringEscapeMalformed:
                    Result::Ok unit
                _:
                    Result::Err "string literal error was not StringEscapeMalformed"
        Result::Ok _argument:
            Result::Err "malformed string escape unexpectedly decoded"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "\"line\\nnext\" \"tab\\tend\" \"carriage\\rend\" \"slash\\\\tail\" \"say\\\"hi\" \"nul\\0end\" \"A\\x42\" \"\\b\" \"\\f\" \"\\'\" \"\\xG1\""
    match lex_all source:
        Result::Ok tokens:
            let str_ty %SelfhostTypeId selfhost_type_id_new 52
            let t0 %SelfhostToken string_token_at &tokens 0
            let t1 %SelfhostToken string_token_at &tokens 1
            let t2 %SelfhostToken string_token_at &tokens 2
            let t3 %SelfhostToken string_token_at &tokens 3
            let t4 %SelfhostToken string_token_at &tokens 4
            let t5 %SelfhostToken string_token_at &tokens 5
            let t6 %SelfhostToken string_token_at &tokens 6
            let t7 %SelfhostToken string_token_at &tokens 7
            let t8 %SelfhostToken string_token_at &tokens 8
            let t9 %SelfhostToken string_token_at &tokens 9
            let t10 %SelfhostToken string_token_at &tokens 10
            let item0 %SelfhostExprPrefixItem string_item_for t0 0
            let item1 %SelfhostExprPrefixItem string_item_for t1 1
            let item2 %SelfhostExprPrefixItem string_item_for t2 2
            let item3 %SelfhostExprPrefixItem string_item_for t3 3
            let item4 %SelfhostExprPrefixItem string_item_for t4 4
            let item5 %SelfhostExprPrefixItem string_item_for t5 5
            let item6 %SelfhostExprPrefixItem string_item_for t6 6
            let item7 %SelfhostExprPrefixItem string_item_for t7 7
            let item8 %SelfhostExprPrefixItem string_item_for t8 8
            let item9 %SelfhostExprPrefixItem string_item_for t9 9
            let item10 %SelfhostExprPrefixItem string_item_for t10 10
            let decoded_newline %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item0 0 1 str_ty
            let decoded_tab %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item1 1 2 str_ty
            let decoded_carriage %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item2 2 3 str_ty
            let decoded_backslash %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item3 3 4 str_ty
            let decoded_quote %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item4 4 5 str_ty
            let decoded_nul %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item5 5 6 str_ty
            let decoded_hex %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item6 6 7 str_ty
            let unsupported_backspace %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item7 7 8 str_ty
            let unsupported_form_feed %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item8 8 9 str_ty
            let unsupported_single_quote %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item9 9 10 str_ty
            let malformed %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item10 10 11 str_ty
            let checks1:
                checks0
                |> checks_push string_payload_value_is decoded_newline "line\nnext"
                |> checks_push string_payload_value_is decoded_tab "tab\tend"
                |> checks_push string_payload_value_is decoded_carriage "carriage\rend"
                |> checks_push string_payload_value_is decoded_backslash "slash\\tail"
                |> checks_push string_payload_value_is decoded_quote "say\"hi"
                |> checks_push string_payload_value_is decoded_nul "nul\0end"
                |> checks_push string_payload_value_is decoded_hex "AB"
                |> checks_push string_payload_error_is_unsupported unsupported_backspace
                |> checks_push string_payload_error_is_unsupported unsupported_form_feed
                |> checks_push string_payload_error_is_unsupported unsupported_single_quote
                |> checks_push string_payload_error_is_malformed malformed
            v::free tokens
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## literal_i32_radix_payload_decode

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
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

fn int_token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    unwrap v::get tokens idx

fn int_item_for %fn SelfhostToken fn i32 SelfhostExprPrefixItem \token\token_index:
    SelfhostExprPrefixItem SelfhostExprPrefixItemKind::IntLiteral token_index field::get token "span"

fn i32_payload_value_is %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError fn i32 Result unit str \result\expected:
    match result:
        Result::Ok argument:
            match field::get argument "kind":
                SelfhostCheckedArgumentKind::I32Literal value:
                    check_eq_i32 expected value
                _:
                    Result::Err "checked argument was not I32Literal"
        Result::Err _error:
            Result::Err "i32 literal payload decode failed"

fn i32_payload_error_is_invalid %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError Result unit str \result:
    match result:
        Result::Err error:
            match error.kind:
                SelfhostLiteralArgumentErrorKind::I32Invalid:
                    Result::Ok unit
                _:
                    Result::Err "i32 literal error was not I32Invalid"
        Result::Ok _argument:
            Result::Err "invalid i32 literal unexpectedly decoded"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "42 0x2a 0X2A 2147483647 2147483648 0x 0x80000000"
    match lex_all source:
        Result::Ok tokens:
            let i32_ty %SelfhostTypeId selfhost_type_id_new 53
            let t0 %SelfhostToken int_token_at &tokens 0
            let t1 %SelfhostToken int_token_at &tokens 1
            let t2 %SelfhostToken int_token_at &tokens 2
            let t3 %SelfhostToken int_token_at &tokens 3
            let t4 %SelfhostToken int_token_at &tokens 4
            let t5 %SelfhostToken int_token_at &tokens 5
            let t6 %SelfhostToken int_token_at &tokens 6
            let item0 %SelfhostExprPrefixItem int_item_for t0 0
            let item1 %SelfhostExprPrefixItem int_item_for t1 1
            let item2 %SelfhostExprPrefixItem int_item_for t2 2
            let item3 %SelfhostExprPrefixItem int_item_for t3 3
            let item4 %SelfhostExprPrefixItem int_item_for t4 4
            let item5 %SelfhostExprPrefixItem int_item_for t5 5
            let item6 %SelfhostExprPrefixItem int_item_for t6 6
            let decimal %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item0 0 1 i32_ty
            let hex_lower %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item1 1 2 i32_ty
            let hex_upper %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item2 2 3 i32_ty
            let max_i32 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item3 3 4 i32_ty
            let decimal_overflow %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item4 4 5 i32_ty
            let empty_hex %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item5 5 6 i32_ty
            let hex_overflow %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_checked_with_source &tokens source item6 6 7 i32_ty
            let checks1:
                checks0
                |> checks_push i32_payload_value_is decimal 42
                |> checks_push i32_payload_value_is hex_lower 42
                |> checks_push i32_payload_value_is hex_upper 42
                |> checks_push i32_payload_value_is max_i32 2147483647
                |> checks_push i32_payload_error_is_invalid decimal_overflow
                |> checks_push i32_payload_error_is_invalid empty_hex
                |> checks_push i32_payload_error_is_invalid hex_overflow
            v::free tokens
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
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
