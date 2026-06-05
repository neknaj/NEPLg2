# NEPLg2 self-host prefix expression input

## builds_flat_expression_prefix_items_from_plain_call

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "std/test" as *

fn check_item %fn &SelfhostExprPrefixList fn i32 fn SelfhostExprPrefixItemKind Result unit str \list\idx\expected:
    match selfhost_expr_prefix_list_get list idx:
        Option::Some item:
            if selfhost_expr_prefix_item_kind_eq item.kind expected Result::Ok unit Result::Err "expr prefix item kind mismatch"
        Option::None:
            Result::Err "expr prefix item missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "add 1 2"
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 0 3 source_span_new_unchecked 0 0 7
            match selfhost_expr_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    let checks1 checks_push checks0 check_eq_i32 3 selfhost_expr_prefix_list_len &list
                    let checks2 checks_push checks1 check_item &list 0 SelfhostExprPrefixItemKind::NamedValue
                    let checks3 checks_push checks2 check_item &list 1 SelfhostExprPrefixItemKind::IntLiteral
                    let checks4 checks_push checks3 check_item &list 2 SelfhostExprPrefixItemKind::IntLiteral
                    selfhost_expr_prefix_list_free list
                    v::free tokens
                    let shown checks_print_report checks4
                    checks_exit_code shown
                Result::Err _e:
                    v::free tokens
                    let checks1 checks_push checks0 Result::Err "expr prefix build failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lex failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## preserves_type_ascription_and_function_value_marker

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
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "std/test" as *

fn check_item %fn &SelfhostExprPrefixList fn i32 fn SelfhostExprPrefixItemKind Result unit str \list\idx\expected:
    match selfhost_expr_prefix_list_get list idx:
        Option::Some item:
            if selfhost_expr_prefix_item_kind_eq item.kind expected Result::Ok unit Result::Err "expr prefix item kind mismatch"
        Option::None:
            Result::Err "expr prefix item missing"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "%fn i32 i32 memo_call @function add"
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 0 8 source_span_new_unchecked 0 0 34
            match selfhost_expr_prefix_list_from_syntax_range &tokens range:
                Result::Ok list:
                    let checks1 checks_push checks0 check_eq_i32 8 selfhost_expr_prefix_list_len &list
                    let checks2 checks_push checks1 check_item &list 0 SelfhostExprPrefixItemKind::TypeAnnotationMarker
                    let checks3 checks_push checks2 check_item &list 1 SelfhostExprPrefixItemKind::FunctionTypeMarker
                    let checks4 checks_push checks3 check_item &list 5 SelfhostExprPrefixItemKind::AtMarker
                    let checks5 checks_push checks4 check_item &list 7 SelfhostExprPrefixItemKind::NamedValue
                    selfhost_expr_prefix_list_free list
                    v::free tokens
                    let shown checks_print_report checks5
                    checks_exit_code shown
                Result::Err _e:
                    v::free tokens
                    let checks1 checks_push checks0 Result::Err "expr prefix build failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "lex failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_non_expression_start_and_legacy_grouping_token

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "std/test" as *

fn expect_error_kind %impure fn Result SelfhostExprPrefixList SelfhostExprPrefixBuildError impure fn SelfhostExprPrefixBuildErrorKind Result unit str \result\expected:
    match result:
        Result::Err err:
            if selfhost_expr_prefix_build_error_kind_eq err.kind expected Result::Ok unit Result::Err "unexpected error kind"
        Result::Ok list:
            selfhost_expr_prefix_list_free list
            Result::Err "expected expr prefix build error"

fn check_source_error %impure fn str impure fn i32 impure fn i32 impure fn SelfhostExprPrefixBuildErrorKind Result unit str \source\token_count\span_end\expected:
    match lex_all source:
        Result::Ok tokens:
            let range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 0 token_count source_span_new_unchecked 0 0 span_end
            let result %Result SelfhostExprPrefixList SelfhostExprPrefixBuildError selfhost_expr_prefix_list_from_syntax_range &tokens range
            let checked %Result unit str expect_error_kind result expected
            v::free tokens
            checked
        Result::Err _diag:
            Result::Err "lex failed"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let checks1 checks_push checks0 check_source_error "void" 1 4 SelfhostExprPrefixBuildErrorKind::MissingExpressionStart
    let checks2 checks_push checks1 check_source_error "add (1)" 4 7 SelfhostExprPrefixBuildErrorKind::InvalidToken
    let shown checks_print_report checks2
    checks_exit_code shown
```

## builds_prefix_items_from_parser_function_body_range

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/ast/prefix_expr" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn check_item %fn &SelfhostExprPrefixList fn i32 fn SelfhostExprPrefixItemKind Result unit str \list\idx\expected:
    match selfhost_expr_prefix_list_get list idx:
        Option::Some item:
            if selfhost_expr_prefix_item_kind_eq item.kind expected Result::Ok unit Result::Err "expr prefix item kind mismatch"
        Option::None:
            Result::Err "expr prefix item missing"

fn check_prefix_from_body %impure fn &Vec SelfhostToken impure fn SelfhostModuleDeclarationBody Result TestReport str \tokens\body:
    match selfhost_expr_prefix_list_from_syntax_range tokens body.first_expression:
        Result::Ok list:
            let checks0 checks_new
            let checks1 checks_push checks0 check_eq_i32 3 selfhost_expr_prefix_list_len &list
            let checks2 checks_push checks1 check_item &list 0 SelfhostExprPrefixItemKind::NamedValue
            let checks3 checks_push checks2 check_item &list 1 SelfhostExprPrefixItemKind::IntLiteral
            let checks4 checks_push checks3 check_item &list 2 SelfhostExprPrefixItemKind::IntLiteral
            selfhost_expr_prefix_list_free list
            Result::Ok checks4
        Result::Err _e:
            Result::Err "expr prefix build from parser body range failed"

fn main %impure fn void i32 \void:
    let source %str "fn main %fn void i32 \\void:\n    add 1 2\n"
    match lex_all source:
        Result::Ok tokens:
            match selfhost_parse_module_tokens source &tokens:
                Result::Ok ast:
                    let item %SelfhostModuleItem item_at &ast 0
                    match item.declaration_body:
                        Option::Some body:
                            match check_prefix_from_body &tokens body:
                                Result::Ok checks:
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let shown checks_print_report checks
                                    checks_exit_code shown
                                Result::Err e:
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let checks checks_push checks_new Result::Err e
                                    let shown checks_print_report checks
                                    checks_exit_code shown
                        Option::None:
                            selfhost_module_ast_free ast
                            v::free tokens
                            let checks checks_push checks_new Result::Err "parser did not attach declaration body evidence"
                            let shown checks_print_report checks
                            checks_exit_code shown
                Result::Err _diag:
                    v::free tokens
                    let checks checks_push checks_new Result::Err "module parser failed"
                    let shown checks_print_report checks
                    checks_exit_code shown
        Result::Err _diag:
            let checks checks_push checks_new Result::Err "lex failed"
            let shown checks_print_report checks
            checks_exit_code shown
```
