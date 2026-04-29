# NEPLg2 self-host import spec

## extracts_import_specs_from_module_ast

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as v
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/import_spec" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "neplg2/core/syntax/parser/module_parser" as *
#import "std/test" as *

fn spec_at <(&Vec<SelfhostImportSpec>,i32)->SelfhostImportSpec> (specs, idx):
    unwrap<SelfhostImportSpec> v::get_ref<SelfhostImportSpec> specs idx

fn main <()*>i32> ():
    let source <str> "#import \"core/result\" as *\n#import \"std/test\" as test\nfn main <()->i32> ():\n    0\n"
    let checks0 checks_new
    match selfhost_parse_module_source source:
        Result::Ok ast:
            match selfhost_module_import_specs &ast:
                Result::Ok specs:
                    let spec_len <i32> v::len_ref<SelfhostImportSpec> &specs
                    let first <SelfhostImportSpec> spec_at &specs 0
                    let second <SelfhostImportSpec> spec_at &specs 1
                    let checks1 checks_push checks0 check_eq_i32 2 spec_len
                    let checks2 checks_push checks1 check_str_eq "core/result" first.path
                    let checks3 checks_push checks2 check_str_eq "*" first.alias
                    let checks4 checks_push checks3 check selfhost_import_spec_is_wildcard first
                    let checks5 checks_push checks4 check_str_eq "std/test" second.path
                    let checks6 checks_push checks5 check_str_eq "test" second.alias
                    let checks7 checks_push checks6 check not selfhost_import_spec_is_wildcard second
                    let checks8 checks_push checks7 check_eq_i32 0 first.span.file_id
                    v::free<SelfhostImportSpec> specs
                    selfhost_module_ast_free ast
                    let shown checks_print_report checks8
                    checks_exit_code shown
                Result::Err _diag:
                    selfhost_module_ast_free ast
                    let checks1 checks_push checks0 Result<(),str>::Err "import spec extraction returned Err"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result<(),str>::Err "parser returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_missing_import_path_quote

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/import_spec" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 checks_new
    let span <SelfhostSourceSpan> source_span_new 3 0 7
    match selfhost_import_spec_parse_lexeme span "#import":
        Result::Ok _spec:
            let checks1 checks_push checks0 Result<(),str>::Err "malformed import was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let checks1 checks_push checks0 check_str_eq "parser.import.path_quote_expected" selfhost_diag_code_name diag.code
            let checks2 checks_push checks1 check_str_eq "import directive requires a quoted path" diag.message
            let shown checks_print_report checks2
            checks_exit_code shown
```

## reports_trailing_text_after_alias

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/import_spec" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 checks_new
    let span <SelfhostSourceSpan> source_span_new 4 0 32
    match selfhost_import_spec_parse_lexeme span "#import \"core/result\" as * extra":
        Result::Ok _spec:
            let checks1 checks_push checks0 Result<(),str>::Err "trailing import text was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let checks1 checks_push checks0 check_str_eq "parser.import.trailing_text" selfhost_diag_code_name diag.code
            let checks2 checks_push checks1 check_str_eq "import directive has trailing text after alias" diag.message
            let shown checks_print_report checks2
            checks_exit_code shown
```
