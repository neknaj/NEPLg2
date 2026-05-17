# NEPLg2 self-host import spec

## parses_import_specs_from_lexemes

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

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/import_spec" as *
#import "std/test" as *
#import "core/math" as *

fn main <()*>i32> ():
    let checks0 checks_new
    match selfhost_import_spec_parse_lexeme source_span_new 0 0 26 "#import \"core/result\" as *":
        Result::Ok first:
            match selfhost_import_spec_parse_lexeme source_span_new 0 27 54 "#import \"std/test\" as test":
                Result::Ok second:
                    let first_span <SelfhostSourceSpan> selfhost_import_spec_span first
                    let first_wildcard <bool> selfhost_import_spec_is_wildcard first
                    let second_wildcard <bool> selfhost_import_spec_is_wildcard second
                    let first_path <str> selfhost_import_spec_path "#import \"core/result\" as *" first
                    let first_alias <str> selfhost_import_spec_alias "#import \"core/result\" as *" first
                    let second_path <str> selfhost_import_spec_path "#import \"std/test\" as test" second
                    let second_alias <str> selfhost_import_spec_alias "#import \"std/test\" as test" second
                    let checks1 checks_push checks0 check_str_eq "core/result" first_path
                    let checks2 checks_push checks1 check_str_eq "*" first_alias
                    let checks3 checks_push checks2 check first_wildcard
                    let checks4 checks_push checks3 check_str_eq "std/test" second_path
                    let checks5 checks_push checks4 check_str_eq "test" second_alias
                    let checks6 checks_push checks5 check not second_wildcard
                    let checks7 checks_push checks6 check_eq_i32 0 first_span.file_id
                    let shown checks_print_report checks7
                    checks_exit_code shown
                Result::Err _diag:
                    let checks1 checks_push checks0 Result<(),str>::Err "second import spec parse returned Err"
                    let shown checks_print_report checks1
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result<(),str>::Err "first import spec parse returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_missing_import_path_quote

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
