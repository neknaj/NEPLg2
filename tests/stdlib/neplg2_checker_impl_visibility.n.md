# NEPLg2 self-host checker impl visibility

## rejects_public_impl_declaration_header

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
#import "neplg2/core/syntax/parser/module_parser" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let source %str "pub impl Show for i32:\n    fn show <(i32)->i32> (x):\n        x\n"
    let checks0 checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            match selfhost_check_module_ast &ast:
                Result::Err diag:
                    let checks1 checks_push checks0 check_str_eq "checker.module.declaration_header_invalid" selfhost_diag_code_name diag.code
                    selfhost_module_ast_free ast
                    let shown checks_print_report checks1
                    checks_exit_code shown
                Result::Ok _summary:
                    selfhost_module_ast_free ast
                    let checks1 checks_push checks0 Result<unit,str>::Err "checker accepted public impl declaration header"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result<unit,str>::Err "parser returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
