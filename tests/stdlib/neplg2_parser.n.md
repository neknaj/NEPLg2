# tests/stdlib/neplg2_parser.n.md

## parses_raw_backend_blocks_into_module_items

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
    ##: [16] ok
    ##: [17] ok
    ##: [18] ok
    ##: [19] ok
    ##: [20] ok
    ##: [21] ok
    ##: [22] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *
#import "core/field" as *
#import "core/math" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn check_item %impure fn TestReport impure fn &SelfhostModuleAst impure fn i32 impure fn str impure fn str TestReport \checks\ast\idx\expected_kind\expected_lexeme:
    let item %SelfhostModuleItem item_at ast idx
    let kind_name %str selfhost_module_item_kind_name item.kind
    let lexeme %str item.lexeme
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn check_type_annotation_range %fn SelfhostSyntaxRange Result unit str \syntax_range:
    match syntax_range:
        SelfhostSyntaxRange::Range range:
            if:
                and:
                    eq range.first_token 4
                    and:
                        eq range.token_count 6
                        and and eq range.span.start 15 eq range.span.end 33 source_span_is_valid range.span
                then:
                    Result::Ok unit
                else:
                    Result::Err "expected function type annotation token range"
        SelfhostSyntaxRange::Empty:
            Result::Err "expected function type annotation range"

fn check_lambda_header_range %fn SelfhostSyntaxRange Result unit str \syntax_range:
    match syntax_range:
        SelfhostSyntaxRange::Range range:
            if:
                and:
                    eq range.first_token 10
                    and:
                        eq range.token_count 4
                        and and eq range.span.start 34 eq range.span.end 38 source_span_is_valid range.span
                then:
                    Result::Ok unit
                else:
                    Result::Err "expected function lambda header token range"
        SelfhostSyntaxRange::Empty:
            Result::Err "expected function lambda header range"

fn check_body_envelope_range %fn SelfhostSyntaxRange Result unit str \syntax_range:
    match syntax_range:
        SelfhostSyntaxRange::Range range:
            if:
                and:
                    eq range.first_token 17
                    and:
                        eq range.token_count 20
                        and and eq range.span.start 44 eq range.span.end 193 source_span_is_valid range.span
                then:
                    Result::Ok unit
                else:
                    Result::Err "expected function body envelope token range"
        SelfhostSyntaxRange::Empty:
            Result::Err "expected function body envelope range"

fn check_body_first_expression_range %fn SelfhostSyntaxRange Result unit str \syntax_range:
    match syntax_range:
        SelfhostSyntaxRange::Range range:
            if:
                and:
                    eq range.first_token 17
                    and:
                        eq range.token_count 1
                        and and eq range.span.start 44 eq range.span.end 60 source_span_is_valid range.span
                then:
                    Result::Ok unit
                else:
                    Result::Err "expected function body first expression token range"
        SelfhostSyntaxRange::Empty:
            Result::Err "expected function body first expression range"

fn check_function_declaration_body %fn SelfhostModuleItem Result unit str \item:
    match item.declaration_body:
        Option::Some body:
            match check_body_envelope_range body.envelope:
                Result::Ok _unit:
                    check_body_first_expression_range body.first_expression
                Result::Err e:
                    Result::Err e
        Option::None:
            Result::Err "expected parser declaration body evidence"

fn check_function_declaration_header %fn SelfhostModuleItem Result unit str \item:
    match item.declaration:
        Option::Some header:
            match header.kind:
                SelfhostModuleDeclarationKind::Function:
                    match header.visibility:
                        SelfhostModuleDeclarationVisibility::Private:
                            match header.head:
                                Option::Some head:
                                    match head.kind:
                                        SelfhostModuleDeclarationHeadKind::Name:
                                            if:
                                                and eq header.header_span.start 8 eq header.header_span.end 39
                                                then:
                                                    match check_type_annotation_range header.type_annotation:
                                                        Result::Ok _unit:
                                                            check_lambda_header_range header.lambda_header
                                                        Result::Err e:
                                                            Result::Err e
                                                else:
                                                    Result::Err "expected current function declaration header span"
                                        SelfhostModuleDeclarationHeadKind::TypeLabel:
                                            Result::Err "expected function name head"
                                Option::None:
                                    Result::Err "expected declaration head"
                        SelfhostModuleDeclarationVisibility::Public:
                            Result::Err "expected private declaration"
                SelfhostModuleDeclarationKind::Struct:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Enum:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Trait:
                    Result::Err "expected function declaration"
                SelfhostModuleDeclarationKind::Impl:
                    Result::Err "expected function declaration"
        Option::None:
            Result::Err "expected parser declaration header evidence"

fn main %impure fn void i32 \void:
    let source %str "//: doc\nfn add %fn i32 fn i32 i32 \\a\\b:\n    #if[target=wasm]\n    #wasm:\n        local.get 0\n        local.get 1\n    #if[target=llvm]\n    #llvmir:\n        %0 = add i32 %a, %b\n        ret i32 %0\n"
    let checks0 checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            let item_len %i32 selfhost_module_ast_len &ast
            let checks1 checks_push checks0 check_eq_i32 10 item_len
            let checks2 check_item checks1 &ast 0 "DocComment" "//: doc"
            let checks3 check_item checks2 &ast 1 "FunctionDecl" "fn add %fn i32 fn i32 i32 \\a\\b:"
            let checks4 check_item checks3 &ast 2 "IfTargetDirective" "#if[target=wasm]"
            let checks5 check_item checks4 &ast 3 "WasmBlock" "#wasm:"
            let checks6 check_item checks5 &ast 4 "WasmText" "local.get 0"
            let checks7 check_item checks6 &ast 5 "WasmText" "local.get 1"
            let checks8 check_item checks7 &ast 6 "IfTargetDirective" "#if[target=llvm]"
            let checks9 check_item checks8 &ast 7 "LlvmIrBlock" "#llvmir:"
            let checks10 check_item checks9 &ast 8 "LlvmIrText" "%0 = add i32 %a, %b"
            let checks11 check_item checks10 &ast 9 "LlvmIrText" "ret i32 %0"
            let checks12 checks_push checks11 check_function_declaration_header item_at &ast 1
            let checks13 checks_push checks12 check_function_declaration_body item_at &ast 1
            selfhost_module_ast_free ast
            let shown checks_print_report checks13
            checks_exit_code shown
        Result::Err diag:
            let _msg %str diag.message
            let checks1 checks_push checks0 Result::Err "parser returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_legacy_grouping_and_angle_syntax

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

#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn expect_parse_err_code_span %impure fn Result SelfhostModuleAst SelfhostDiagnostic impure fn str impure fn i32 impure fn i32 Result unit str \r\expected\span_start\span_end:
    match r:
        Result::Err diag:
            match check_str_eq expected selfhost_diag_code_name diag.code:
                Result::Ok _unit:
                    match diag.primary_label:
                        Option::Some label:
                            let span %SelfhostSourceSpan label.span
                            let start_ok %bool eq span.start span_start
                            let end_ok %bool eq span.end span_end
                            if:
                                and start_ok end_ok
                                then:
                                    Result::Ok unit
                                else:
                                    Result::Err "legacy diagnostic primary label span mismatch"
                        Option::None:
                            Result::Err "expected legacy diagnostic primary label"
                Result::Err e:
                    Result::Err e
        Result::Ok ast:
            selfhost_module_ast_free ast
            Result::Err "expected parser diagnostic"

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let legacy_angle %Result SelfhostModuleAst SelfhostDiagnostic selfhost_parse_module_source "fn add <(i32,i32)->i32> (a,b):\n    add a b\n"
    let checks1 checks_push checks0 expect_parse_err_code_span legacy_angle "parser.syntax.legacy_token" 7 8
    let legacy_grouping %Result SelfhostModuleAst SelfhostDiagnostic selfhost_parse_module_source "fn main %fn void i32 \\void:\n    (add 1 2)\n"
    let checks2 checks_push checks1 expect_parse_err_code_span legacy_grouping "parser.syntax.legacy_token" 32 33
    let shown checks_print_report checks2
    checks_exit_code shown
```

## rejects_invalid_raw_and_dedent_states

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn parser_test_token %fn TokenKind fn i32 fn i32 SelfhostToken \kind\start\end:
    selfhost_token_new kind source_span_new_unchecked 0 start end

fn expect_parse_err_code %impure fn Result SelfhostModuleAst SelfhostDiagnostic impure fn str Result unit str \r\expected:
    match r:
        Result::Err diag:
            check_str_eq expected selfhost_diag_code_name diag.code
        Result::Ok ast:
            selfhost_module_ast_free ast
            Result::Err "expected parser diagnostic"

fn raw_unclosed_tokens %impure fn void Vec SelfhostToken \void:
    let tokens0 %Vec SelfhostToken unwrap_ok v::new
    let tokens1 %Vec SelfhostToken unwrap_ok v::push tokens0 parser_test_token TokenKind::DirWasm 0 6
    let tokens2 %Vec SelfhostToken unwrap_ok v::push tokens1 parser_test_token TokenKind::Indent 7 7
    let tokens3 %Vec SelfhostToken unwrap_ok v::push tokens2 parser_test_token TokenKind::WasmText 8 19
    unwrap_ok v::push tokens3 parser_test_token TokenKind::Eof 19 19

fn extra_dedent_tokens %impure fn void Vec SelfhostToken \void:
    let tokens0 %Vec SelfhostToken unwrap_ok v::new
    let tokens1 %Vec SelfhostToken unwrap_ok v::push tokens0 parser_test_token TokenKind::Dedent 0 0
    unwrap_ok v::push tokens1 parser_test_token TokenKind::Eof 0 0

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let pending_result %Result SelfhostModuleAst SelfhostDiagnostic selfhost_parse_module_source "#wasm:\n"
    let checks1 checks_push checks0 expect_parse_err_code pending_result "parser.raw_block.expected_indent"
    let raw_source %str "#wasm:\n    local.get 0"
    let raw_tokens %Vec SelfhostToken raw_unclosed_tokens
    let raw_result %Result SelfhostModuleAst SelfhostDiagnostic selfhost_parse_module_tokens raw_source &raw_tokens
    v::free raw_tokens
    let checks2 checks_push checks1 expect_parse_err_code raw_result "parser.raw_block.unclosed"
    let dedent_tokens %Vec SelfhostToken extra_dedent_tokens
    let dedent_result %Result SelfhostModuleAst SelfhostDiagnostic selfhost_parse_module_tokens "" &dedent_tokens
    v::free dedent_tokens
    let checks3 checks_push checks2 expect_parse_err_code dedent_result "parser.indent.invalid_dedent"
    let shown checks_print_report checks3
    checks_exit_code shown
```
