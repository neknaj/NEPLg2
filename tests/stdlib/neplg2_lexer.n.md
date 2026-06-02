# NEPLg2 self-host lexer

## lexes_directive_function_signature_and_integer

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "#entry main\nfn main %impure fn void i32 \\void:\n    42\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 18 token_len
            let checks2 check_token checks1 source &tokens 0 "DirEntry" "main"
            let checks3 check_token checks2 source &tokens 2 "KwFn" "fn"
            let checks4 check_token checks3 source &tokens 3 "Ident" "main"
            let checks5 check_token checks4 source &tokens 7 "VoidMarker" "void"
            let checks6 check_token checks5 source &tokens 10 "VoidMarker" "void"
            let checks7 check_token checks6 source &tokens 13 "Indent" ""
            let checks8 check_token checks7 source &tokens 14 "IntLiteral" "42"
            let checks9 check_token checks8 source &tokens 16 "Dedent" ""
            let checks10 check_token checks9 source &tokens 17 "Eof" ""
            free tokens
            let shown checks_print_report checks10
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## emits_nested_indent_dedent

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "a:\n    b:\n        c\n    d\nz"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 16 token_len
            let checks2 check_token checks1 source &tokens 3 "Indent" ""
            let checks3 check_token checks2 source &tokens 7 "Indent" ""
            let checks4 check_token checks3 source &tokens 8 "Ident" "c"
            let checks5 check_token checks4 source &tokens 10 "Dedent" ""
            let checks6 check_token checks5 source &tokens 11 "Ident" "d"
            let checks7 check_token checks6 source &tokens 13 "Dedent" ""
            let checks8 check_token checks7 source &tokens 14 "Ident" "z"
            let checks9 check_token checks8 source &tokens 15 "Eof" ""
            free tokens
            let shown checks_print_report checks9
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lex_all_with_file_id_sets_token_and_error_spans

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
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all_with_file_id "#entry main\n" 7:
        Result::Ok tokens:
            let token %SelfhostToken token_at &tokens 0
            let span %SelfhostSourceSpan field::get token "span"
            let checks1 checks_push checks0 check_eq_i32 7 field::get span "file_id"
            free tokens
            match lex_all_with_file_id "a:\n   b\n" 11:
                Result::Ok bad_tokens:
                    free bad_tokens
                    let checks2 checks_push checks1 Result::Err "invalid indentation was accepted"
                    let shown checks_print_report checks2
                    checks_exit_code shown
                Result::Err diag:
                    let err_span %SelfhostSourceSpan field::get diag "span"
                    let checks2 checks_push checks1 check_eq_i32 11 field::get err_span "file_id"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result::Err "file_id lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## honors_indent_directive_width

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

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "#indent 2\nfn:\n  1\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 10 token_len
            let checks2 check_token checks1 source &tokens 0 "DirIndentWidth" "#indent 2"
            let checks3 check_token checks2 source &tokens 5 "Indent" ""
            let checks4 check_token checks3 source &tokens 6 "IntLiteral" "1"
            let checks5 check_token checks4 source &tokens 8 "Dedent" ""
            let checks6 check_token checks5 source &tokens 9 "Eof" ""
            free tokens
            let shown checks_print_report checks6
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_level_mismatch

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

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all "a:\n    b\n  c\n":
        Result::Ok tokens:
            free tokens
            let checks1 checks_push checks0 Result::Err "indent mismatch was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name %str selfhost_lexer_diag_code_name field::get diag "code"
            let span %SelfhostSourceSpan field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.indent.level_mismatch" code_name
                |> checks_push check_eq_i32 9 field::get span "start"
                |> checks_push check_eq_i32 9 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_width_mismatch

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

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all "a:\n   b\n":
        Result::Ok tokens:
            free tokens
            let checks1 checks_push checks0 Result::Err "indent width mismatch was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name %str selfhost_lexer_diag_code_name field::get diag "code"
            let span %SelfhostSourceSpan field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.indent.level_mismatch" code_name
                |> checks_push check_eq_i32 3 field::get span "start"
                |> checks_push check_eq_i32 3 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## skips_comments_and_reports_unexpected_character

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

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all "name // skip this\n$":
        Result::Ok tokens:
            free tokens
            let checks1 checks_push checks0 Result::Err "unexpected character was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name %str selfhost_lexer_diag_code_name field::get diag "code"
            let span %SelfhostSourceSpan field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.token.unknown" code_name
                |> checks_push check_eq_i32 18 field::get span "start"
                |> checks_push check_eq_i32 19 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_string

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

#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all "\"abc":
        Result::Ok tokens:
            free tokens
            let checks1 checks_push checks0 Result::Err "unterminated string was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name %str selfhost_lexer_diag_code_name field::get diag "code"
            let span %SelfhostSourceSpan field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.string.unterminated" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_char_literal

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

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn main %impure fn void i32 \void:
    let checks0 checks_new
    let source %str "'\\n' 'a'"
    match lex_all source:
        Result::Ok tokens:
            let t0 %SelfhostToken token_at &tokens 0
            let t1 %SelfhostToken token_at &tokens 1
            let checks1:
                checks0
                |> checks_push check_str_eq "CharLiteral" token_kind_name field::get t0 "kind"
                |> checks_push check_str_eq "'\\n'" selfhost_token_lexeme source t0
                |> checks_push check_str_eq "CharLiteral" token_kind_name field::get t1 "kind"
                |> checks_push check_str_eq "'a'" selfhost_token_lexeme source t1
            free tokens
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_char

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

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn void i32 \void:
    let checks0 checks_new
    match lex_all "'abc":
        Result::Ok tokens:
            free tokens
            let checks1 checks_push checks0 Result::Err "unterminated char was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name %str selfhost_lexer_diag_code_name field::get diag "code"
            let span %SelfhostSourceSpan field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.char.invalid" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## matches_rust_token_names_for_directives_keywords_and_literals

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
    ##: [23] ok
    ##: [24] ok
    ##: [25] ok
    ##: [26] ok
    ##: [27] ok
    ##: [28] ok
    ##: [29] ok
    ##: [30] ok
    ##: [31] ok
    ##: [32] ok
    ##: [33] ok
    ##: [34] ok
    ##: [35] ok
    ##: [36] ok
    ##: [37] ok
    ##: [38] ok
    ##: [39] ok
    ##: [40] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "#target core\n#import \"std/test\" as *\n#use \"std/prelude\"\n#if[target=core]\n#if[profile=debug]\n#capability io\n#prelude \"std/prelude\"\n#no_prelude\n#intrinsic \"unreachable\" <> ()\nfn main %fn void i32 \\void:\n    let mut x 0x2a;\n    set x 1.5;\n    if cond true then 'a' else \"s\"\n    Result::Ok x\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 60 token_len
            let checks2 check_token checks1 source &tokens 0 "DirTarget" "#target core"
            let checks3 check_token checks2 source &tokens 2 "DirImport" "#import \"std/test\" as *"
            let checks4 check_token checks3 source &tokens 4 "DirUse" "#use \"std/prelude\""
            let checks5 check_token checks4 source &tokens 6 "DirIfTarget" "#if[target=core]"
            let checks6 check_token checks5 source &tokens 8 "DirIfProfile" "#if[profile=debug]"
            let checks7 check_token checks6 source &tokens 10 "DirCapability" "#capability io"
            let checks8 check_token checks7 source &tokens 12 "DirPrelude" "#prelude \"std/prelude\""
            let checks9 check_token checks8 source &tokens 14 "DirNoPrelude" "#no_prelude"
            let checks10 check_token checks9 source &tokens 16 "DirIntrinsic" "#intrinsic"
            let checks11 check_token checks10 source &tokens 17 "StringLiteral" "\"unreachable\""
            let checks12 check_token checks11 source &tokens 23 "KwFn" "fn"
            let checks13 check_token checks12 source &tokens 27 "VoidMarker" "void"
            let checks14 check_token checks13 source &tokens 30 "VoidMarker" "void"
            let checks15 check_token checks14 source &tokens 34 "KwLet" "let"
            let checks16 check_token checks15 source &tokens 35 "KwMut" "mut"
            let checks17 check_token checks16 source &tokens 42 "FloatLiteral" "1.5"
            let checks18 check_token checks17 source &tokens 45 "KwIf" "if"
            let checks19 check_token checks18 source &tokens 47 "BoolLiteral" "true"
            let checks20 check_token checks19 source &tokens 54 "PathSep" "::"
            let checks21 check_token checks20 source &tokens 58 "Dedent" ""
            let checks22 check_token checks21 source &tokens 59 "Eof" ""
            free tokens
            let shown checks_print_report checks22
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_doc_comment_and_mlstr_tokens

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
    ##: [8] ok
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "//: module doc\n/// item doc\n##: text\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 7 token_len
            let checks2 check_token checks1 source &tokens 0 "DocComment" "//: module doc"
            let checks3 check_token checks2 source &tokens 2 "DocComment" "/// item doc"
            let checks4 check_token checks3 source &tokens 4 "MlstrLine" "##: text"
            let checks5 check_token checks4 source &tokens 6 "Eof" ""
            free tokens
            let shown checks_print_report checks5
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_raw_wasm_and_llvmir_block_text

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
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
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/syntax/lexer" as *
#import "neplg2/core/syntax/token" as *
#import "std/test" as *
#import "core/field" as *

fn token_at %fn &Vec SelfhostToken fn i32 SelfhostToken \tokens\idx:
    let found %Option SelfhostToken get tokens idx
    unwrap found

fn check_token %impure fn TestReport impure fn str impure fn &Vec SelfhostToken impure fn i32 impure fn str impure fn str TestReport \checks\source\tokens\idx\expected_kind\expected_lexeme:
    let token %SelfhostToken token_at tokens idx
    let kind_name %str token_kind_name field::get token "kind"
    let lexeme %str selfhost_token_lexeme source token
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main %impure fn void i32 \void:
    let source %str "//: doc\n##: text\n#wasm:\n    local.get 0\n#llvmir:\n    ret i32 0\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len %i32 len &tokens
            let checks1 checks_push checks0 check_eq_i32 17 token_len
            let checks2 check_token checks1 source &tokens 4 "DirWasm" "#wasm:"
            let checks3 check_token checks2 source &tokens 6 "Indent" ""
            let checks4 check_token checks3 source &tokens 7 "WasmText" "local.get 0"
            let checks5 check_token checks4 source &tokens 9 "Dedent" ""
            let checks6 check_token checks5 source &tokens 10 "DirLlvmIr" "#llvmir:"
            let checks7 check_token checks6 source &tokens 12 "Indent" ""
            let checks8 check_token checks7 source &tokens 13 "LlvmIrText" "ret i32 0"
            let checks9 check_token checks8 source &tokens 15 "Dedent" ""
            let checks10 check_token checks9 source &tokens 16 "Eof" ""
            free tokens
            let shown checks_print_report checks10
            checks_exit_code shown
        Result::Err diag:
            let _msg %str field::get diag "message"
            let checks1 checks_push checks0 Result::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
