# block_single_line.rs 由来の doctest

このファイルは Rust テスト `block_single_line.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## block_sl_basic_literal

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_basic_literal\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block literal\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block 10
    let report:
        test_report_new "block_sl_basic_literal"
        |> test_report_push assert_eq_i32 "single-line block literal" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_basic_arithmetic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_basic_arithmetic\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block arithmetic\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block add 1 2
    let report:
        test_report_new "block_sl_basic_arithmetic"
        |> test_report_push assert_eq_i32 "single-line block arithmetic" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_with_let

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_with_let\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block let\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block let x 10; x
    let report:
        test_report_new "block_sl_with_let"
        |> test_report_push assert_eq_i32 "single-line block let" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_multiple_stmts

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_multiple_stmts\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block multiple statements\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block let x 1; let y 2; add x y
    let report:
        test_report_new "block_sl_multiple_stmts"
        |> test_report_push assert_eq_i32 "single-line block multiple statements" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_nested

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_nested\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested single-line block\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block block 5
    let report:
        test_report_new "block_sl_nested"
        |> test_report_push assert_eq_i32 "nested single-line block" 5 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_nested_in_multiline

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_nested_in_multiline\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block in multiline block\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block:
        block 10
    let report:
        test_report_new "block_sl_nested_in_multiline"
        |> test_report_push assert_eq_i32 "single-line block in multiline block" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_arg_position

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_arg_position\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block argument\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> add 1 block 2
    let report:
        test_report_new "block_sl_arg_position"
        |> test_report_push assert_eq_i32 "single-line block argument" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_arg_position_complex

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_arg_position_complex\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block complex arguments\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    // add (block 1 (block 2)) と正しく解釈される
    let actual <i32> add block 1 block 2
    let report:
        test_report_new "block_sl_arg_position_complex"
        |> test_report_push assert_eq_i32 "single-line block complex arguments" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_if_branch

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_if_branch\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block if branch\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    // blockのルールによると if true (block 1 else (block 2)) と解釈されるため誤り
    let actual <i32> if true block 1 else block 2
    let report:
        test_report_new "block_sl_if_branch"
        |> test_report_push assert_eq_i32 "single-line block if branch" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_while_body

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_while_body\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block while body\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut i 0
    // while lt i 5 (block set i add i 1) と解釈され、正しい
    while lt i 5 block set i add i 1
    let report:
        test_report_new "block_sl_while_body"
        |> test_report_push assert_eq_i32 "single-line block while body" 5 i
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_semicolon_unit

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_semicolon_unit\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block semicolon unit\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    // block returns unit, so we return 0 explicitly
    block 1;
    let actual <i32> 0
    let report:
        test_report_new "block_sl_semicolon_unit"
        |> test_report_push assert_eq_i32 "single-line block semicolon unit" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_shadowing

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_shadowing\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block shadowing\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let x 1
    let y block let x 2; x
    // y should be 2, outer x is 1
    let actual <i32> if eq x 1 y 0
    let report:
        test_report_new "block_sl_shadowing"
        |> test_report_push assert_eq_i32 "single-line block shadowing" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_mutation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_mutation\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block mutation\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let mut x 1
    block set x 2
    let report:
        test_report_new "block_sl_mutation"
        |> test_report_push assert_eq_i32 "single-line block mutation" 2 x
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_type_annotated

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_type_annotated\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block type annotation\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> <i32> block 10
    let report:
        test_report_new "block_sl_type_annotated"
        |> test_report_push assert_eq_i32 "single-line block type annotation" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_tuple_element


このテストは「単行ブロック（`block ...`）が式として評価され、その結果をタプル要素として扱える」ことを確認する意図です。
ただし、元のテストはタプルの旧リテラル記法 `(a, b)` と、数値フィールドアクセス `t.1` を用いていました。todo.md の方針ではこれらは廃止対象なので、
タプル生成は新記法 `Tuple:` に、要素取得は `core/field` の `get` に置き換えました。
これにより、テストの主旨（単行ブロック式の評価・タプル要素化）は維持したまま、仕様の最新方針に整合させています。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_tuple_element\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block tuple element\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/field" as *
#import "std/test" as *

fn main <()*>i32> ():
    let t Tuple:
        block 1
        block 2
    let actual <i32> get t 1
    let report:
        test_report_new "block_sl_tuple_element"
        |> test_report_push assert_eq_i32 "single-line block tuple element" 2 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_pipe_source

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_pipe_source\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block pipe source\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block 1 |> add 2
    let report:
        test_report_new "block_sl_pipe_source"
        |> test_report_push assert_eq_i32 "single-line block pipe source" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_match_arm

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_match_arm\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block match arm\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/mem" as *
#import "std/test" as *

enum E: A

fn main <()*>i32> ():
    let actual <i32> match E::A:
        A: block 10
    let report:
        test_report_new "block_sl_match_arm"
        |> test_report_push assert_eq_i32 "single-line block match arm" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_trailing_comment

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_trailing_comment\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block trailing comment\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block 1 // comment
    let report:
        test_report_new "block_sl_trailing_comment"
        |> test_report_push assert_eq_i32 "single-line block trailing comment" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_empty_ish

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_empty_ish\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"single-line block unit then return\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    block ()
    let actual <i32> 0
    let report:
        test_report_new "block_sl_empty_ish"
        |> test_report_push assert_eq_i32 "single-line block unit then return" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_deeply_nested

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"block_sl_deeply_nested\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"deeply nested single-line block\" expected=\"99\" actual=\"99\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "std/test" as *

fn main <()*>i32> ():
    let actual <i32> block block block 99
    let report:
        test_report_new "block_sl_deeply_nested"
        |> test_report_push assert_eq_i32 "deeply nested single-line block" 99 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## block_sl_single_line_block_cannot_contain_multiline_if

neplg2:test[compile_fail]
diag_code: parser.token.unexpected
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    block if:
        true
        1
        2
```
