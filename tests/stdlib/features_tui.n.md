# features/tui facade

このファイルは `features/tui` の利用者向け入口と、named struct 化した TUI API の最小回帰を固定します。
TTY がない環境でも成立する helper と、`get_terminal_size` の field access を分けて確認します。

## features_tui_facade_reexports_text_helpers

[目的/もくてき]:
- 利用者が `platforms/wasix/tui` を直接 import せず、`features/tui` から helper を使えることを[確/たし]かめます。
- TUI facade が単なる path alias ではなく、既存 helper 群の公式入口として機能していることを固定します。

neplg2:test
stdout: "ab  ::xxx\n"
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "std/stdio" as *

fn main %impure fn void unit \void:
    let left %str tui::line_pad_to_cols "ab" 4;
    let right %str tui::repeat_text "x" 3;
    print left;
    print "::";
    print right;
    println "";
```

## features_tui_terminal_size_uses_named_fields

[目的/もくてき]:
- `get_terminal_size` の戻り値が `.Pair` ではなく、named field を持つ struct として扱えることを[確/たし]かめます。
- TTY が取れない環境でも `0,0` を返して壊れず、`"cols"` / `"rows"` access が成立することを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"features_tui_terminal_size_uses_named_fields\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "core/math" as *
#import "core/field" as *
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let size tui::get_terminal_size;
    let cols %i32 get size "cols";
    let rows %i32 get size "rows";
    if:
        or lt cols 0 lt rows 0
        then:
            1
        else:
            0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "features_tui_terminal_size_uses_named_fields"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## features_tui_buffer_new_initializes_string_lines

[目的/もくてき]:
- 行バッファが `str` 行スロットを型付き store で初期化し、facade 経由の import だけで compile fail しないことを固定します。
- `buffer_new` / `buffer_set_line` / `buffer_free` の最小経路を通し、空行初期化と後続の `str` 書き込みが同じ型で扱われることを確かめます。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"features_tui_buffer_new_initializes_string_lines\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "std/test" as *

fn run_case %impure fn void i32 \void:
    let b %i32 tui::buffer_new 8 2;
    tui::buffer_set_line b 1 "ready";
    tui::buffer_free b;
    0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test_report_new "features_tui_buffer_new_initializes_string_lines"
        |> test_report_push assert_eq_i32 "return value" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## features_tui_box_helpers_clamp_narrow_widths

[目的/もくてき]:
- `line_top` / `line_bottom` / `line_box` / `line_box_styled` が `cols` 0, 1, 2, 3 を内部で安全に扱うことを固定します。
- 本文が内側幅より長いときも、呼び出し側が事前に clip しなくてよいことを確かめます。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"features_tui_box_helpers_clamp_narrow_widths\" count=15 failed=0\nassertion index=0 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=2 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=3 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=4 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=5 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=6 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=7 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=8 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=9 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=10 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=11 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"a\" actual=\"a\" message=\"\"\nassertion index=12 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"ab\" actual=\"ab\" message=\"\"\nassertion index=13 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\" actual=\"\" message=\"\"\nassertion index=14 status=ok kind=str_eq label=\"assert_str_eq\" expected=\"\u001b[31m\u001b[44ma\u001b[0m\" actual=\"\u001b[31m\u001b[44ma\u001b[0m\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "std/stdio" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let box_style %AnsiTextStyle ansi_color_pair_style AnsiColor::Red AnsiColor::Blue;
    let checks:
        test_report_new "features_tui_box_helpers_clamp_narrow_widths"
        |> test_report_push assert_str_eq "" tui::line_top 0
        |> test_report_push assert_str_eq "┌" tui::line_top 1
        |> test_report_push assert_str_eq "┌┐" tui::line_top 2
        |> test_report_push assert_str_eq "┌─┐" tui::line_top 3
        |> test_report_push assert_str_eq "" tui::line_bottom 0
        |> test_report_push assert_str_eq "└" tui::line_bottom 1
        |> test_report_push assert_str_eq "└┘" tui::line_bottom 2
        |> test_report_push assert_str_eq "└─┘" tui::line_bottom 3
        |> test_report_push assert_str_eq "" tui::line_box "abc" 0
        |> test_report_push assert_str_eq "│" tui::line_box "abc" 1
        |> test_report_push assert_str_eq "││" tui::line_box "abc" 2
        |> test_report_push assert_str_eq "│a│" tui::line_box "abc" 3
        |> test_report_push assert_str_eq "│ab│" tui::line_box "abcd" 4
        |> test_report_push assert_str_eq "││" tui::line_box_styled box_style "abc" 2
        |> test_report_push assert_str_eq "│\x1b[31m\x1b[44ma\x1b[0m│" tui::line_box_styled box_style "abc" 3
    let shown test_report_print_stdout checks;
    test_report_exit_code shown
```

## features_tui_color_helpers_use_typed_ansi_style

[目的/もくてき]:
- TUI の色付き文字列 helper が raw `i32` code ではなく `AnsiColor` / `AnsiTextStyle` を使うことを固定します。
- style 文字列生成と直接出力の両方が `std/stdio/ansi` の enum/match 変換を通ることを確かめます。

neplg2:test
stdout: "\u001b[31m\u001b[44mx\u001b[0m\n\u001b[32mfg\u001b[44mbg\u001b[0m\n"
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "std/stdio" as *

fn main %impure fn void unit \void:
    let inline_style %AnsiTextStyle ansi_color_pair_style AnsiColor::Red AnsiColor::Blue;
    print tui::style_text inline_style "x";
    println "";
    tui::set_fg_color AnsiColor::Green;
    print "fg";
    tui::set_bg_color AnsiColor::Blue;
    print "bg";
    tui::reset_color;
    println "";
```
