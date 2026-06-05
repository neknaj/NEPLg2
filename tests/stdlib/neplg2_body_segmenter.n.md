# NEPLg2 self-host body segmenter

## splits_multiple_top_level_expression_lines

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
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/body_segmenter" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn segment_at %fn &SelfhostBodySegmentList fn i32 SelfhostBodySegment \segments\idx:
    let segment_opt %Option SelfhostBodySegment selfhost_body_segment_list_get segments idx
    unwrap segment_opt

fn check_segment_kind %fn &SelfhostBodySegmentList fn i32 fn SelfhostBodySegmentKind Result unit str \segments\idx\expected:
    let segment %SelfhostBodySegment segment_at segments idx
    if selfhost_body_segment_kind_eq segment.kind expected Result::Ok unit Result::Err "unexpected body segment kind"

fn main %impure fn void i32 \void:
    let source %str "fn main %fn void i32 \\void:\n    let x %i32 1\n    add x 2\n"
    match lex_all source:
        Result::Ok tokens:
            match selfhost_parse_module_tokens source &tokens:
                Result::Ok ast:
                    let item %SelfhostModuleItem item_at &ast 0
                    match item.declaration_body:
                        Option::Some body:
                            match selfhost_body_segment_list_from_envelope &tokens body.envelope:
                                Result::Ok segments:
                                    let checks0 checks_new
                                    let checks1 checks_push checks0 check_eq_i32 2 selfhost_body_segment_list_len &segments
                                    let checks2 checks_push checks1 check_segment_kind &segments 0 SelfhostBodySegmentKind::ExpressionLine
                                    let checks3 checks_push checks2 check_segment_kind &segments 1 SelfhostBodySegmentKind::ExpressionLine
                                    selfhost_body_segment_list_free segments
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let shown checks_print_report checks3
                                    checks_exit_code shown
                                Result::Err _e:
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let checks checks_push checks_new Result::Err "body segmenter failed"
                                    let shown checks_print_report checks
                                    checks_exit_code shown
                        Option::None:
                            selfhost_module_ast_free ast
                            v::free tokens
                            let checks checks_push checks_new Result::Err "parser did not attach declaration body"
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

## keeps_single_line_body_as_one_expression_segment

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
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/body_segmenter" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn segment_at %fn &SelfhostBodySegmentList fn i32 SelfhostBodySegment \segments\idx:
    let segment_opt %Option SelfhostBodySegment selfhost_body_segment_list_get segments idx
    unwrap segment_opt

fn check_segment_kind %fn &SelfhostBodySegmentList fn i32 fn SelfhostBodySegmentKind Result unit str \segments\idx\expected:
    let segment %SelfhostBodySegment segment_at segments idx
    if selfhost_body_segment_kind_eq segment.kind expected Result::Ok unit Result::Err "unexpected body segment kind"

fn main %impure fn void i32 \void:
    let source %str "fn main %fn void i32 \\void: add 1 2\n"
    match lex_all source:
        Result::Ok tokens:
            match selfhost_parse_module_tokens source &tokens:
                Result::Ok ast:
                    let item %SelfhostModuleItem item_at &ast 0
                    match item.declaration_body:
                        Option::Some body:
                            match selfhost_body_segment_list_from_envelope &tokens body.envelope:
                                Result::Ok segments:
                                    let checks0 checks_new
                                    let checks1 checks_push checks0 check_eq_i32 1 selfhost_body_segment_list_len &segments
                                    let checks2 checks_push checks1 check_segment_kind &segments 0 SelfhostBodySegmentKind::ExpressionLine
                                    selfhost_body_segment_list_free segments
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let shown checks_print_report checks2
                                    checks_exit_code shown
                                Result::Err _e:
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let checks checks_push checks_new Result::Err "body segmenter failed"
                                    let shown checks_print_report checks
                                    checks_exit_code shown
                        Option::None:
                            selfhost_module_ast_free ast
                            v::free tokens
                            let checks checks_push checks_new Result::Err "parser did not attach declaration body"
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

## keeps_nested_block_as_block_intro_segment

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

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/parser/body_segmenter" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn item_at %fn &SelfhostModuleAst fn i32 SelfhostModuleItem \ast\idx:
    let item_opt %Option SelfhostModuleItem selfhost_module_ast_get ast idx
    unwrap item_opt

fn segment_at %fn &SelfhostBodySegmentList fn i32 SelfhostBodySegment \segments\idx:
    let segment_opt %Option SelfhostBodySegment selfhost_body_segment_list_get segments idx
    unwrap segment_opt

fn check_segment_kind %fn &SelfhostBodySegmentList fn i32 fn SelfhostBodySegmentKind Result unit str \segments\idx\expected:
    let segment %SelfhostBodySegment segment_at segments idx
    if selfhost_body_segment_kind_eq segment.kind expected Result::Ok unit Result::Err "unexpected body segment kind"

fn check_nonempty_range %fn SelfhostSyntaxRange Result unit str \range:
    match range:
        SelfhostSyntaxRange::Range _items:
            Result::Ok unit
        SelfhostSyntaxRange::Empty:
            Result::Err "expected nonempty nested body range"

fn check_nested_segments %impure fn &Vec SelfhostToken impure fn SelfhostSyntaxRange Result TestReport str \tokens\body_range:
    match selfhost_body_segment_list_from_envelope tokens body_range:
        Result::Ok nested:
            let checks0 checks_new
            let checks1 checks_push checks0 check_eq_i32 3 selfhost_body_segment_list_len &nested
            let checks2 checks_push checks1 check_segment_kind &nested 0 SelfhostBodySegmentKind::ExpressionLine
            let checks3 checks_push checks2 check_segment_kind &nested 1 SelfhostBodySegmentKind::BlockIntro
            let checks4 checks_push checks3 check_segment_kind &nested 2 SelfhostBodySegmentKind::BlockIntro
            selfhost_body_segment_list_free nested
            Result::Ok checks4
        Result::Err _e:
            Result::Err "nested body segmenter failed"

fn main %impure fn void i32 \void:
    let source %str "fn main %fn void i32 \\void:\n    if:\n        true\n        then:\n            1\n        else:\n            2\n    add 1 2\n"
    match lex_all source:
        Result::Ok tokens:
            match selfhost_parse_module_tokens source &tokens:
                Result::Ok ast:
                    let item %SelfhostModuleItem item_at &ast 0
                    match item.declaration_body:
                        Option::Some body:
                            match selfhost_body_segment_list_from_envelope &tokens body.envelope:
                                Result::Ok segments:
                                    let first %SelfhostBodySegment segment_at &segments 0
                                    match check_nested_segments &tokens first.body:
                                        Result::Ok nested_checks:
                                            let checks1 checks_push nested_checks check_eq_i32 2 selfhost_body_segment_list_len &segments
                                            let checks2 checks_push checks1 check_segment_kind &segments 0 SelfhostBodySegmentKind::BlockIntro
                                            let checks3 checks_push checks2 check_segment_kind &segments 1 SelfhostBodySegmentKind::ExpressionLine
                                            let checks4 checks_push checks3 check_nonempty_range first.body
                                            selfhost_body_segment_list_free segments
                                            selfhost_module_ast_free ast
                                            v::free tokens
                                            let shown checks_print_report checks4
                                            checks_exit_code shown
                                        Result::Err e:
                                            selfhost_body_segment_list_free segments
                                            selfhost_module_ast_free ast
                                            v::free tokens
                                            let checks checks_push checks_new Result::Err e
                                            let shown checks_print_report checks
                                            checks_exit_code shown
                                Result::Err _e:
                                    selfhost_module_ast_free ast
                                    v::free tokens
                                    let checks checks_push checks_new Result::Err "body segmenter failed"
                                    let shown checks_print_report checks
                                    checks_exit_code shown
                        Option::None:
                            selfhost_module_ast_free ast
                            v::free tokens
                            let checks checks_push checks_new Result::Err "parser did not attach declaration body"
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
