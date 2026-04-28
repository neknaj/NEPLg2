# NEPLg2 self-host lexer

## lexes_directive_function_signature_and_integer

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get_ref<SelfhostToken> tokens idx

fn check_token <(Vec<Result<(),str>>, &Vec<SelfhostToken>, i32, str, str)*>Vec<Result<(),str>>> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 <Vec<Result<(),str>>> checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "#entry main\nfn main <()*>i32> ():\n    42\n"
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len_ref<SelfhostToken> &tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 21 token_len
            let checks2 <Vec<Result<(),str>>> check_token checks1 &tokens 0 "hash" "#"
            let checks3 <Vec<Result<(),str>>> check_token checks2 &tokens 1 "identifier" "entry"
            let checks4 <Vec<Result<(),str>>> check_token checks3 &tokens 2 "identifier" "main"
            let checks5 <Vec<Result<(),str>>> check_token checks4 &tokens 9 "effect_arrow" "*>"
            let checks6 <Vec<Result<(),str>>> check_token checks5 &tokens 16 "indent" ""
            let checks7 <Vec<Result<(),str>>> check_token checks6 &tokens 17 "int_literal" "42"
            let checks8 <Vec<Result<(),str>>> check_token checks7 &tokens 19 "dedent" ""
            let checks9 <Vec<Result<(),str>>> check_token checks8 &tokens 20 "eof" ""
            free<SelfhostToken> tokens
            let shown <Vec<Result<(),str>>> checks_print_report checks9
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## emits_nested_indent_dedent

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get_ref<SelfhostToken> tokens idx

fn check_token <(Vec<Result<(),str>>, &Vec<SelfhostToken>, i32, str, str)*>Vec<Result<(),str>>> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 <Vec<Result<(),str>>> checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "a:\n    b:\n        c\n    d\nz"
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len_ref<SelfhostToken> &tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 16 token_len
            let checks2 <Vec<Result<(),str>>> check_token checks1 &tokens 3 "indent" ""
            let checks3 <Vec<Result<(),str>>> check_token checks2 &tokens 7 "indent" ""
            let checks4 <Vec<Result<(),str>>> check_token checks3 &tokens 8 "identifier" "c"
            let checks5 <Vec<Result<(),str>>> check_token checks4 &tokens 10 "dedent" ""
            let checks6 <Vec<Result<(),str>>> check_token checks5 &tokens 11 "identifier" "d"
            let checks7 <Vec<Result<(),str>>> check_token checks6 &tokens 13 "dedent" ""
            let checks8 <Vec<Result<(),str>>> check_token checks7 &tokens 14 "identifier" "z"
            let checks9 <Vec<Result<(),str>>> check_token checks8 &tokens 15 "eof" ""
            free<SelfhostToken> tokens
            let shown <Vec<Result<(),str>>> checks_print_report checks9
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## honors_indent_directive_width

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get_ref<SelfhostToken> tokens idx

fn check_token <(Vec<Result<(),str>>, &Vec<SelfhostToken>, i32, str, str)*>Vec<Result<(),str>>> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 <Vec<Result<(),str>>> checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "#indent 2\nfn:\n  1\n"
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len_ref<SelfhostToken> &tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 check_eq_i32 10 token_len
            let checks2 <Vec<Result<(),str>>> check_token checks1 &tokens 0 "dir_indent_width" "#indent 2"
            let checks3 <Vec<Result<(),str>>> check_token checks2 &tokens 5 "indent" ""
            let checks4 <Vec<Result<(),str>>> check_token checks3 &tokens 6 "int_literal" "1"
            let checks5 <Vec<Result<(),str>>> check_token checks4 &tokens 8 "dedent" ""
            let checks6 <Vec<Result<(),str>>> check_token checks5 &tokens 9 "eof" ""
            free<SelfhostToken> tokens
            let shown <Vec<Result<(),str>>> checks_print_report checks6
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_level_mismatch

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "a:\n    b\n  c\n":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "indent mismatch was accepted"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> lex_error_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "lex.invalid_indentation" code_name
                |> checks_push check_eq_i32 9 field::get span "start"
                |> checks_push check_eq_i32 9 field::get span "end"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_width_mismatch

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "a:\n   b\n":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "indent width mismatch was accepted"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> lex_error_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "lex.invalid_indentation" code_name
                |> checks_push check_eq_i32 3 field::get span "start"
                |> checks_push check_eq_i32 3 field::get span "end"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## skips_comments_and_reports_unexpected_character

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "name // skip this\n@":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "unexpected character was accepted"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> lex_error_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "lex.unexpected_char" code_name
                |> checks_push check_eq_i32 18 field::get span "start"
                |> checks_push check_eq_i32 19 field::get span "end"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_string

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "\"abc":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "unterminated string was accepted"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> lex_error_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "lex.unterminated_string" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## lexes_char_literal

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get_ref<SelfhostToken> tokens idx

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "'\\n' 'a'":
        Result::Ok tokens:
            let t0 <SelfhostToken> token_at &tokens 0
            let t1 <SelfhostToken> token_at &tokens 1
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "char_literal" token_kind_name field::get t0 "kind"
                |> checks_push check_str_eq "'\\n'" field::get t0 "lexeme"
                |> checks_push check_str_eq "char_literal" token_kind_name field::get t1 "kind"
                |> checks_push check_str_eq "'a'" field::get t1 "lexeme"
            free<SelfhostToken> tokens
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_char

neplg2:test
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match lex_all "'abc":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "unterminated char was accepted"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> lex_error_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_str_eq "lex.unterminated_char" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```
