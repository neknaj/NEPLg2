# NEPLg2 self-host checker

## summarizes_module_items_with_typed_kind_match

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

#import "core/result" as *
#import "neplg2/core/check/checker" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let source %str "//: doc\n#entry main\n#target std\n#import \"core/result\" as *\nfn main <()->i32> ():\n    0\nstruct Pair:\nenum Maybe:\ntrait Show:\nimpl Show for Pair:\n#wasm:\n    i32.const 0\n"
    let mut checks checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            match selfhost_check_module_ast &ast:
                Result::Ok summary:
                    set checks checks_push checks check_eq_i32 10 selfhost_module_check_summary_item_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_doc_comment_count summary
                    set checks checks_push checks check_eq_i32 4 selfhost_module_check_summary_directive_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_entry_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_target_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_import_count summary
                    set checks checks_push checks check_eq_i32 4 selfhost_module_check_summary_declaration_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_function_count summary
                    set checks checks_push checks check_eq_i32 2 selfhost_module_check_summary_type_declaration_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_impl_count summary
                    set checks checks_push checks check_eq_i32 1 selfhost_module_check_summary_raw_text_count summary
                    selfhost_module_ast_free ast
                    let shown checks_print_report checks
                    checks_exit_code shown
                Result::Err _diag:
                    selfhost_module_ast_free ast
                    set checks checks_push checks Result<unit,str>::Err "checker returned Err"
                    let shown checks_print_report checks
                    checks_exit_code shown
        Result::Err _diag:
            set checks checks_push checks Result<unit,str>::Err "parser returned Err"
            let shown checks_print_report checks
            checks_exit_code shown
```

## rejects_duplicate_singleton_directives

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

#import "core/result" as *
#import "neplg2/core/check/checker" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *

fn check_duplicate_directive %impure fn SelfhostModuleItemKind Result unit str \kind:
    match selfhost_module_ast_new:
        Result::Ok ast0:
            let span1 %SelfhostSourceSpan source_span_new 0 0 7
            let span2 %SelfhostSourceSpan source_span_new 0 8 15
            let item1 %SelfhostModuleItem selfhost_module_item_new kind span1 "first"
            match selfhost_module_ast_push ast0 item1:
                Result::Ok ast1:
                    let item2 %SelfhostModuleItem selfhost_module_item_new kind span2 "second"
                    match selfhost_module_ast_push ast1 item2:
                        Result::Ok ast2:
                            match selfhost_check_module_ast &ast2:
                                Result::Err diag:
                                    let result %Result unit str check_str_eq "checker.module.directive_duplicate" selfhost_diag_code_name diag.code
                                    selfhost_module_ast_free ast2
                                    result
                                Result::Ok _summary:
                                    selfhost_module_ast_free ast2
                                    Result<unit,str>::Err "duplicate singleton directive was accepted"
                        Result::Err _e:
                            Result<unit,str>::Err "second module AST push failed"
                Result::Err _e:
                    Result<unit,str>::Err "first module AST push failed"
        Result::Err _e:
            Result<unit,str>::Err "module AST allocation failed"

fn main %impure fn unit i32 \unit:
    let checks0 checks_new
    let checks1 checks_push checks0 check_duplicate_directive SelfhostModuleItemKind::EntryDirective
    let checks2 checks_push checks1 check_duplicate_directive SelfhostModuleItemKind::TargetDirective
    let shown checks_print_report checks2
    checks_exit_code shown
```

## rejects_raw_text_without_matching_raw_block

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/check/checker" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let checks0 checks_new
    match selfhost_module_ast_new:
        Result::Ok ast0:
            let span %SelfhostSourceSpan source_span_new 0 0 12
            let item %SelfhostModuleItem selfhost_module_item_new SelfhostModuleItemKind::WasmText span "i32.const 0"
            match selfhost_module_ast_push ast0 item:
                Result::Ok ast:
                    match selfhost_check_module_ast &ast:
                        Result::Err diag:
                            let checks1 checks_push checks0 check_str_eq "checker.module.raw_text_without_block" selfhost_diag_code_name diag.code
                            selfhost_module_ast_free ast
                            let shown checks_print_report checks1
                            checks_exit_code shown
                        Result::Ok _summary:
                            selfhost_module_ast_free ast
                            let checks1 checks_push checks0 Result<unit,str>::Err "checker accepted orphan raw text"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<unit,str>::Err "module AST push failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<unit,str>::Err "module AST allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_declaration_items_without_parser_header_evidence

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/check/checker" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let checks0 checks_new
    match selfhost_module_ast_new:
        Result::Ok ast0:
            let span %SelfhostSourceSpan source_span_new 0 0 24
            let item %SelfhostModuleItem selfhost_module_item_new SelfhostModuleItemKind::FunctionDecl span "fn main <()->i32> \():"
            match selfhost_module_ast_push ast0 item:
                Result::Ok ast:
                    match selfhost_check_module_ast &ast:
                        Result::Err diag:
                            let checks1 checks_push checks0 check_str_eq "checker.module.declaration_header_missing" selfhost_diag_code_name diag.code
                            selfhost_module_ast_free ast
                            let shown checks_print_report checks1
                            checks_exit_code shown
                        Result::Ok _summary:
                            selfhost_module_ast_free ast
                            let checks1 checks_push checks0 Result<unit,str>::Err "checker accepted declaration item without parser header"
                            let shown checks_print_report checks1
                            checks_exit_code shown
                Result::Err _e:
                    let checks1 checks_push checks0 Result<unit,str>::Err "module AST push failed"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<unit,str>::Err "module AST allocation failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
