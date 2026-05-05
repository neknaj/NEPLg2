# string.rs 由来の doctest

このファイルは Rust テスト `string.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。

## test_string_literal_single_line_type

以前は `compile_ok` により「型として受理されるか」だけを見ていましたが、実行できる形にして内容（改行の扱い）まで確認します。
単行文字列リテラルはエスケープを解釈し、`\\n` は改行として出力されることを期待します。

neplg2:test[normalize_newlines]
stdout: "hello\nworld!"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    // 単行文字列の \n が実行時に改行として扱われることを確認する
    print "hello\nworld!"
```

## test_string_literal_mlstr_type

以前は `compile_ok` で型だけ確認していました。
`mlstr:` が「行間に \\n を挿入し、末尾には挿入しない」仕様どおりに実行時に組み立てられることを、標準出力で確認します。

neplg2:test[normalize_newlines]
stdout: "hello\nworld!"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let b <str> mlstr:
        ##: hello
        ##: world!
    // mlstr の内容がそのまま出力されることを確認する
    print b
```

## test_mlstr_line_separator

以前は `compile_ok` で「書けるか」だけを確認していました。
`mlstr:` が末尾に余計な改行を付けないことを、後続の `"END"` と連結して出力することで確認します（もし末尾に \\n が入ると `"world!\\nEND"` になって不一致になります）。

neplg2:test[normalize_newlines]
stdout: "hello\nworld!END"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let b <str> mlstr:
        ##: hello
        ##: world!
    print b
    print "END"
```

## test_mlstr_raw_no_escape

以前は `compile_ok` で「構文が通る」だけでした。
仕様では `mlstr:` は Raw 文字列であり、`\\n` や `\\t` をエスケープとして解釈しないことが重要です。
そのため、`\\n` と `\\t` を含む内容をそのまま出力し、末尾に `"END"` を付けて検証します。

neplg2:test[normalize_newlines]
stdout: "\\n should be literal backslash-n\nno \\t escape processingEND"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let raw <str> mlstr:
        ##: \n should be literal backslash-n
        ##: no \t escape processing
    print raw
    print "END"
```

## test_single_line_with_escapes

以前は `compile_ok` で「書ける」だけでした。
単行文字列では `\\n` と `\\t` がそれぞれ改行・タブとして解釈されることを、実行時の出力で確認します。

neplg2:test[normalize_newlines]
stdout: "hello\nworld!\ttab"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    // \n と \t が実行時に制御文字として扱われることを確認する
    print "hello\nworld!\ttab"
```

## test_str_no_ownership

以前は `compile_ok` で型だけ確認していました。
`str` が借用ビューとして「値のコピー（ポインタ＋長さのコピー）で扱える」ことは実行結果だけでは完全には検証できませんが、
少なくとも `let b <str> a;` が実行可能で、同じ内容が出力されることを確認します。

neplg2:test
stdout: "static literal"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let a <str> "static literal"
    let b <str> a
    // b が a と同内容を参照できることを確認する
    print b
```

## test_str_lifetime_static

以前は `compile_ok` のみでした。
ここでは `'static` 相当の寿命を持つリテラル `str` をそのまま出力できることを確認します。

neplg2:test
stdout: "hello"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let a <str> "hello"
    print a
```

## test_string_byte_at

`byte_at` が byte index で ASCII byte を返し、範囲外を `None` にすることを確認します。

neplg2:test[normalize_newlines]
exit_code: 0
```neplg2
#target std
#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *

fn main <()* >i32> ():
    let ok0 <bool> match byte_at "AZ" 0:
        Option::Some b:
            eq b 65
        Option::None:
            false
    let ok1 <bool> match byte_at "AZ" 1:
        Option::Some b:
            eq b 90
        Option::None:
            false
    let ok2 <bool> is_none<i32> byte_at "AZ" 2
    if and ok0 and ok1 ok2 0 1
```

## test_string_literal_unicode

以前は `compile_ok` で構文・型だけを見ていました。
UTF-8 文字列（日本語・絵文字）が実行時に欠損せずに出力されることを確認します。

neplg2:test
stdout: "こんにちは世界👋🌍"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let japanese <str> "こんにちは世界"
    let emoji <str> "👋🌍"
    // UTF-8 の連結（連続出力）が崩れないことを確認する
    print japanese
    print emoji
```

## test_mlstr_unicode

以前は `compile_ok` のみでした。
`mlstr:` が UTF-8 の行も正しく保持し、行間に \\n を挿入することを `"END"` 連結で確認します。

neplg2:test[normalize_newlines]
stdout: "こんにちは\n世界END"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let text <str> mlstr:
        ##: こんにちは
        ##: 世界
    print text
    print "END"
```

## test_mlstr_trailing_whitespace

以前は `compile_ok` で「受理されるか」だけを見ていました。
仕様では `mlstr:` は行末の trim を行わないため、行末スペースが保持されることが重要です。
ここでは視認性のために `[` と `]` で囲って出力し、`line1` の後ろに 3 つのスペースが残ることを `stdout:` で確認します。

neplg2:test[normalize_newlines]
stdout: "[line1   \nline2]END"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    let text <str> mlstr:
        ##: line1   
        ##: line2
    print "["
    print text
    print "]"
    print "END"
```

## test_mlstr_missing_prefix_is_error

`mlstr:` 本文は `##: ` で始まる行のみ許可します。
接頭辞のない行を混在させた場合は、パース時にエラーになることを固定します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
fn main <()->i32> ():
    let text <str> mlstr:
        ##: line1
        line2 without prefix
    0
```

## test_string_to_str_implicit_conversion

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
fn foo <(str)->()> (s):
    ()
fn main <()->i32> ():
    foo "hello"
    0
```

## test_str_to_string_implicit_conversion

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
fn foo <(String)->()> (s):
    ()
fn main <()->i32> ():
    foo <str> "hello" // should not work
    0
```

## test_string_to_str_explicit_conversion

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
fn foo <(str)->()> (s):
    ()
fn main <()->i32> ():
    foo <str> "hello"
    0
```

## test_str_to_string_explicit_conversion

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
fn foo <(String)->()> (s):
    ()
fn main <()->i32> ():
    foo <String> "hello" // should not work
    0
```

## test_string_builder_linear_build

neplg2:test[normalize_newlines]
exit_code: 0
stdout: "Checked [ok]\n[0] ok\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *
#import "alloc/string" as *

fn main <()* >i32> ():
    let mut sb <StringBuilder> string_builder_new;
    let mut i <i32> 0;
    while lt i 2000:
        do:
            set sb sb_append sb "a";
            set i add i 1;
    let out <str> sb_build sb;
    let checks:
        checks_new
        |> checks_push assert_eq_i32 2000 len out
    let shown checks_print_report checks
    checks_exit_code shown
```
