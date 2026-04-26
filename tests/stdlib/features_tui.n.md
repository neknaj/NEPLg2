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

fn main <()*>()> ():
    let left <str> tui::line_pad_to_cols "ab" 4;
    let right <str> tui::repeat_text "x" 3;
    print left;
    print "::";
    print right;
    println "";
```

## features_tui_terminal_size_uses_named_fields

[目的/もくてき]:
- `get_terminal_size` の戻り値が `.Pair` ではなく、named field を持つ struct として扱えることを[確/たし]かめます。
- TTY が取れない環境でも `0,0` を返して壊れず、`"cols"` / `"rows"` access が成立することを固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui
#import "core/math" as *

fn main <()*>i32> ():
    let size tui::get_terminal_size;
    let cols <i32> get size "cols";
    let rows <i32> get size "rows";
    if:
        or lt cols 0 lt rows 0
        then:
            1
        else:
            0
```

## features_tui_buffer_new_initializes_string_lines

[目的/もくてき]:
- 行バッファが `str` 行スロットを型付き store で初期化し、facade 経由の import だけで compile fail しないことを固定します。
- `buffer_new` / `buffer_set_line` / `buffer_free` の最小経路を通し、空行初期化と後続の `str` 書き込みが同じ型で扱われることを確かめます。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasix

#import "features/tui" as tui

fn main <()*>i32> ():
    let b <i32> tui::buffer_new 8 2;
    tui::buffer_set_line b 1 "ready";
    tui::buffer_free b;
    0
```
