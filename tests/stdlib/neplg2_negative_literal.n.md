# NEPLg2 selfhost negative numeric literal

## negative_numeric_literal_prefix_and_payload

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
    ##: [11] ok
    ##: [12] ok
    ##: [13] ok
    ##: [14] ok
    ##: [15] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/check/expr/argument_payload" as *
#import "neplg2/core/check/expr/literal_payload" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn item_at %fn &SelfhostExprPrefixList fn i32 SelfhostExprPrefixItem \list\idx:
    unwrap selfhost_expr_prefix_list_get list idx

fn item_kind_is %fn &SelfhostExprPrefixList fn i32 fn SelfhostExprPrefixItemKind Result unit str \list\idx\expected:
    let item %SelfhostExprPrefixItem item_at list idx
    if selfhost_expr_prefix_item_kind_eq item.kind expected Result::Ok unit Result::Err "prefix item kind mismatch"

fn signed_literal_item %fn SelfhostExprPrefixItemKind fn i32 fn SelfhostSourceSpan SelfhostExprPrefixItem \kind\token_index\span:
    selfhost_expr_prefix_item_new kind token_index span

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

fn f32_payload_value_is %fn Result SelfhostCheckedArgument SelfhostLiteralArgumentError fn f32 Result unit str \result\expected:
    match result:
        Result::Ok argument:
            match field::get argument "kind":
                SelfhostCheckedArgumentKind::F32Literal value:
                    check eq value expected
                _:
                    Result::Err "checked argument was not F32Literal"
        Result::Err _error:
            Result::Err "f32 literal payload decode failed"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "-42 -0x2a -2147483648 -2147483649 -1.5 -"
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 0 11 source_span_new_unchecked 0 0 40
            match selfhost_expr_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    let i32_ty %SelfhostTypeId selfhost_type_id_new 53
                    let f32_ty %SelfhostTypeId selfhost_type_id_new 54
                    let neg_decimal_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::IntLiteral 0 source_span_new_unchecked 0 0 3
                    let neg_hex_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::IntLiteral 2 source_span_new_unchecked 0 4 9
                    let min_i32_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::IntLiteral 4 source_span_new_unchecked 0 10 21
                    let overflow_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::IntLiteral 6 source_span_new_unchecked 0 22 33
                    let neg_float_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::FloatLiteral 8 source_span_new_unchecked 0 34 38
                    let neg_decimal %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens source item_at &list 0 item_at &list 1 neg_decimal_item 0 2 i32_ty
                    let neg_hex %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens source item_at &list 2 item_at &list 3 neg_hex_item 2 4 i32_ty
                    let min_i32 %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens source item_at &list 4 item_at &list 5 min_i32_item 4 6 i32_ty
                    let overflow %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens source item_at &list 6 item_at &list 7 overflow_item 6 8 i32_ty
                    let neg_float %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens source item_at &list 8 item_at &list 9 neg_float_item 8 10 f32_ty
                    let expected_f32 %f32 -1.5
                    let checks1 checks_push checks0 check_eq_i32 11 selfhost_expr_prefix_list_len &list
                    let checks2 checks_push checks1 item_kind_is &list 0 SelfhostExprPrefixItemKind::MinusMarker
                    let checks3 checks_push checks2 item_kind_is &list 1 SelfhostExprPrefixItemKind::IntLiteral
                    let checks4 checks_push checks3 item_kind_is &list 2 SelfhostExprPrefixItemKind::MinusMarker
                    let checks5 checks_push checks4 item_kind_is &list 8 SelfhostExprPrefixItemKind::MinusMarker
                    let checks6 checks_push checks5 item_kind_is &list 9 SelfhostExprPrefixItemKind::FloatLiteral
                    let checks7 checks_push checks6 item_kind_is &list 10 SelfhostExprPrefixItemKind::MinusMarker
                    let checks8 checks_push checks7 i32_payload_value_is neg_decimal -42
                    let checks9 checks_push checks8 i32_payload_value_is neg_hex -42
                    let checks10 checks_push checks9 i32_payload_value_is min_i32 -2147483648
                    let checks11 checks_push checks10 i32_payload_error_is_invalid overflow
                    let checks12 checks_push checks11 f32_payload_value_is neg_float expected_f32
                    selfhost_expr_prefix_list_free list
                    v::free tokens
                    let source2 %str "- 42 - 1.5"
                    match lex_all source2:
                        Result::Ok tokens2:
                            let range2 %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 0 4 source_span_new_unchecked 0 0 10
                            match selfhost_expr_prefix_list_from_syntax_range &tokens2 range2:
                                Result::Ok list2:
                                    let spaced_i32_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::IntLiteral 0 source_span_new_unchecked 0 0 4
                                    let spaced_f32_item %SelfhostExprPrefixItem signed_literal_item SelfhostExprPrefixItemKind::FloatLiteral 2 source_span_new_unchecked 0 5 10
                                    let spaced_decimal %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens2 source2 item_at &list2 0 item_at &list2 1 spaced_i32_item 0 2 i32_ty
                                    let spaced_float %Result SelfhostCheckedArgument SelfhostLiteralArgumentError selfhost_literal_argument_negative_checked_with_source &tokens2 source2 item_at &list2 2 item_at &list2 3 spaced_f32_item 2 4 f32_ty
                                    let checks13 checks_push checks12 check_eq_i32 4 selfhost_expr_prefix_list_len &list2
                                    let checks14 checks_push checks13 item_kind_is &list2 0 SelfhostExprPrefixItemKind::MinusMarker
                                    let checks15 checks_push checks14 i32_payload_value_is spaced_decimal -42
                                    let checks16 checks_push checks15 f32_payload_value_is spaced_float expected_f32
                                    selfhost_expr_prefix_list_free list2
                                    v::free tokens2
                                    let shown checks_print_report checks16
                                    checks_exit_code shown
                                Result::Err _e:
                                    v::free tokens2
                                    let checks13 checks_push checks12 Result::Err "spaced negative prefix build failed"
                                    let shown checks_print_report checks13
                                    checks_exit_code shown
                        Result::Err _diag:
                            let checks13 checks_push checks12 Result::Err "spaced negative lexer returned Err"
                            let shown checks_print_report checks13
                            checks_exit_code shown
                Result::Err _e:
                    v::free tokens
                    let checks1 checks_push checks0 Result::Err "prefix build failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
