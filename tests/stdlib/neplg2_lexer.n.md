# NEPLg2 self-host lexer

## lexes_directive_function_signature_and_integer

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "#entry main\nfn main <()*>i32> ():\n    42\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 19 token_len
            let checks2 check_token checks1 &tokens 0 "DirEntry" "main"
            let checks3 check_token checks2 &tokens 2 "KwFn" "fn"
            let checks4 check_token checks3 &tokens 3 "Ident" "main"
            let checks5 check_token checks4 &tokens 7 "Arrow" "*>"
            let checks6 check_token checks5 &tokens 14 "Indent" ""
            let checks7 check_token checks6 &tokens 15 "IntLiteral" "42"
            let checks8 check_token checks7 &tokens 17 "Dedent" ""
            let checks9 check_token checks8 &tokens 18 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks9
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## emits_nested_indent_dedent

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "a:\n    b:\n        c\n    d\nz"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 16 token_len
            let checks2 check_token checks1 &tokens 3 "Indent" ""
            let checks3 check_token checks2 &tokens 7 "Indent" ""
            let checks4 check_token checks3 &tokens 8 "Ident" "c"
            let checks5 check_token checks4 &tokens 10 "Dedent" ""
            let checks6 check_token checks5 &tokens 11 "Ident" "d"
            let checks7 check_token checks6 &tokens 13 "Dedent" ""
            let checks8 check_token checks7 &tokens 14 "Ident" "z"
            let checks9 check_token checks8 &tokens 15 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks9
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lex_all_with_file_id_sets_token_and_error_spans

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all_with_file_id "#entry main\n" 7:
        Result::Ok tokens:
            let token <SelfhostToken> token_at &tokens 0
            let span <SelfhostSourceSpan> field::get token "span"
            let checks1 checks_push checks0 check_eq_i32 7 field::get span "file_id"
            free<SelfhostToken> tokens
            match lex_all_with_file_id "a:\n   b\n" 11:
                Result::Ok bad_tokens:
                    free<SelfhostToken> bad_tokens
                    let checks2 checks_push checks1 Result<(),str>::Err "invalid indentation was accepted"
                    let shown checks_print_report checks2
                    checks_exit_code shown
                Result::Err diag:
                    let err_span <SelfhostSourceSpan> field::get diag "span"
                    let checks2 checks_push checks1 check_eq_i32 11 field::get err_span "file_id"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _diag:
            let checks1 checks_push checks0 Result<(),str>::Err "file_id lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## honors_indent_directive_width

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "#indent 2\nfn:\n  1\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 10 token_len
            let checks2 check_token checks1 &tokens 0 "DirIndentWidth" "#indent 2"
            let checks3 check_token checks2 &tokens 5 "Indent" ""
            let checks4 check_token checks3 &tokens 6 "IntLiteral" "1"
            let checks5 check_token checks4 &tokens 8 "Dedent" ""
            let checks6 check_token checks5 &tokens 9 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks6
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_level_mismatch

neplg2:test
ret: 0
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

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "a:\n    b\n  c\n":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 checks_push checks0 Result<(),str>::Err "indent mismatch was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> selfhost_lexer_diag_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.indent.level_mismatch" code_name
                |> checks_push check_eq_i32 9 field::get span "start"
                |> checks_push check_eq_i32 9 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_indent_width_mismatch

neplg2:test
ret: 0
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

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "a:\n   b\n":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 checks_push checks0 Result<(),str>::Err "indent width mismatch was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> selfhost_lexer_diag_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.indent.level_mismatch" code_name
                |> checks_push check_eq_i32 3 field::get span "start"
                |> checks_push check_eq_i32 3 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## skips_comments_and_reports_unexpected_character

neplg2:test
ret: 0
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

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "name // skip this\n$":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 checks_push checks0 Result<(),str>::Err "unexpected character was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> selfhost_lexer_diag_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.token.unknown" code_name
                |> checks_push check_eq_i32 18 field::get span "start"
                |> checks_push check_eq_i32 19 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_string

neplg2:test
ret: 0
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

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "\"abc":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 checks_push checks0 Result<(),str>::Err "unterminated string was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> selfhost_lexer_diag_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.string.unterminated" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_char_literal

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "'\\n' 'a'":
        Result::Ok tokens:
            let t0 <SelfhostToken> token_at &tokens 0
            let t1 <SelfhostToken> token_at &tokens 1
            let checks1:
                checks0
                |> checks_push check_str_eq "CharLiteral" token_kind_name field::get t0 "kind"
                |> checks_push check_str_eq "'\\n'" field::get t0 "lexeme"
                |> checks_push check_str_eq "CharLiteral" token_kind_name field::get t1 "kind"
                |> checks_push check_str_eq "'a'" field::get t1 "lexeme"
            free<SelfhostToken> tokens
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## reports_unterminated_char

neplg2:test
ret: 0
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

fn main <()*>i32> ():
    let checks0 checks_new
    match lex_all "'abc":
        Result::Ok tokens:
            free<SelfhostToken> tokens
            let checks1 checks_push checks0 Result<(),str>::Err "unterminated char was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err diag:
            let code_name <str> selfhost_lexer_diag_code_name field::get diag "code"
            let span <SelfhostSourceSpan> field::get diag "span"
            let checks1:
                checks0
                |> checks_push check_str_eq "lexer.char.invalid" code_name
                |> checks_push check_eq_i32 0 field::get span "start"
                |> checks_push check_eq_i32 4 field::get span "end"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## matches_rust_token_names_for_directives_keywords_and_literals

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "#target core\n#import \"std/test\" as *\n#use \"std/prelude\"\n#if[target=core]\n#if[profile=debug]\n#capability io\n#prelude \"std/prelude\"\n#no_prelude\n#intrinsic \"unreachable\" <> ()\nfn main <()->i32> ():\n    let mut x 0x2a;\n    set x 1.5;\n    if cond true then 'a' else \"s\"\n    Result::Ok x\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 62 token_len
            let checks2 check_token checks1 &tokens 0 "DirTarget" "#target core"
            let checks3 check_token checks2 &tokens 2 "DirImport" "#import \"std/test\" as *"
            let checks4 check_token checks3 &tokens 4 "DirUse" "#use \"std/prelude\""
            let checks5 check_token checks4 &tokens 6 "DirIfTarget" "#if[target=core]"
            let checks6 check_token checks5 &tokens 8 "DirIfProfile" "#if[profile=debug]"
            let checks7 check_token checks6 &tokens 10 "DirCapability" "#capability io"
            let checks8 check_token checks7 &tokens 12 "DirPrelude" "#prelude \"std/prelude\""
            let checks9 check_token checks8 &tokens 14 "DirNoPrelude" "#no_prelude"
            let checks10 check_token checks9 &tokens 16 "DirIntrinsic" "#intrinsic"
            let checks11 check_token checks10 &tokens 17 "StringLiteral" "\"unreachable\""
            let checks12 check_token checks11 &tokens 23 "KwFn" "fn"
            let checks13 check_token checks12 &tokens 28 "Arrow" "->"
            let checks14 check_token checks13 &tokens 36 "KwLet" "let"
            let checks15 check_token checks14 &tokens 37 "KwMut" "mut"
            let checks16 check_token checks15 &tokens 44 "FloatLiteral" "1.5"
            let checks17 check_token checks16 &tokens 47 "KwIf" "if"
            let checks18 check_token checks17 &tokens 49 "BoolLiteral" "true"
            let checks19 check_token checks18 &tokens 56 "PathSep" "::"
            let checks20 check_token checks19 &tokens 60 "Dedent" ""
            let checks21 check_token checks20 &tokens 61 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks21
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_doc_comment_and_mlstr_tokens

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "//: module doc\n/// item doc\n##: text\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 7 token_len
            let checks2 check_token checks1 &tokens 0 "DocComment" "//: module doc"
            let checks3 check_token checks2 &tokens 2 "DocComment" "/// item doc"
            let checks4 check_token checks3 &tokens 4 "MlstrLine" "##: text"
            let checks5 check_token checks4 &tokens 6 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks5
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## lexes_raw_wasm_and_llvmir_block_text

neplg2:test
ret: 0
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

fn token_at <(&Vec<SelfhostToken>,i32)->SelfhostToken> (tokens, idx):
    unwrap<SelfhostToken> get<SelfhostToken> tokens idx

fn check_token <(TestReport, &Vec<SelfhostToken>, i32, str, str)*>TestReport> (checks, tokens, idx, expected_kind, expected_lexeme):
    let token <SelfhostToken> token_at tokens idx
    let kind_name <str> token_kind_name field::get token "kind"
    let lexeme <str> field::get token "lexeme"
    let checks1 checks_push checks check_str_eq expected_kind kind_name
    checks_push checks1 check_str_eq expected_lexeme lexeme

fn main <()*>i32> ():
    let source <str> "//: doc\n##: text\n#wasm:\n    local.get 0\n#llvmir:\n    ret i32 0\n"
    let checks0 checks_new
    match lex_all source:
        Result::Ok tokens:
            let token_len <i32> len<SelfhostToken> &tokens
            let checks1 checks_push checks0 check_eq_i32 17 token_len
            let checks2 check_token checks1 &tokens 4 "DirWasm" "#wasm:"
            let checks3 check_token checks2 &tokens 6 "Indent" ""
            let checks4 check_token checks3 &tokens 7 "WasmText" "local.get 0"
            let checks5 check_token checks4 &tokens 9 "Dedent" ""
            let checks6 check_token checks5 &tokens 10 "DirLlvmIr" "#llvmir:"
            let checks7 check_token checks6 &tokens 12 "Indent" ""
            let checks8 check_token checks7 &tokens 13 "LlvmIrText" "ret i32 0"
            let checks9 check_token checks8 &tokens 15 "Dedent" ""
            let checks10 check_token checks9 &tokens 16 "Eof" ""
            free<SelfhostToken> tokens
            let shown checks_print_report checks10
            checks_exit_code shown
        Result::Err diag:
            let _msg <str> field::get diag "message"
            let checks1 checks_push checks0 Result<(),str>::Err "lexer returned Err"
            let shown checks_print_report checks1
            checks_exit_code shown
```
