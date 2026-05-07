---
id: ISS-20260507T065612249Z-WASIX-TUI-COLOR-HELPERS-STILL-ACCEPT-1FD08056
title: "WASIX TUI color helpers still accept raw i32 ANSI codes"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/platforms/wasix/tui.nepl, stdlib/std/stdio/ansi.nepl, tests/stdlib/features_tui.n.md, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js"
---

# ISS-20260507T065612249Z-WASIX-TUI-COLOR-HELPERS-STILL-ACCEPT-1FD08056: WASIX TUI color helpers still accept raw i32 ANSI codes

## 概要

stdlib/platforms/wasix/tui.nepl keeps set_fg_color, set_bg_color, and style_text on raw i32 color codes even after std/stdio/ansi was redesigned around AnsiColor and AnsiTextStyle. This leaves TUI color output outside enum/match exhaustiveness checks.

## 対象

- `stdlib/platforms/wasix/tui.nepl, stdlib/std/stdio/ansi.nepl, tests/stdlib/features_tui.n.md, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`

## 根拠

- `stdlib/platforms/wasix/tui.nepl` の `set_fg_color` / `set_bg_color` / `style_text` / `line_box_styled` は raw `i32` ANSI color code を受け取っていた。
- `std/stdio/ansi.nepl` は `AnsiColor` / `AnsiTextStyle` を使う typed API に移行済みであり、TUI だけが enum/match による静的検査対象から外れていた。
- focused `features_tui` compile では、同じ TUI file の `get_tty_state` が「成功 pointer / 失敗時 0」を同じ raw `i32` で返すため、Resource owner checker が `get_terminal_size` の `dealloc_raw state` を `MaybeFreed` と診断することも確認した。

## 問題

stdlib/platforms/wasix/tui.nepl keeps set_fg_color, set_bg_color, and style_text on raw i32 color codes even after std/stdio/ansi was redesigned around AnsiColor and AnsiTextStyle. This leaves TUI color output outside enum/match exhaustiveness checks.

## 影響

TUI callers can pass unsupported numeric color codes, color semantics are duplicated between stdio and TUI, and future ANSI color changes can diverge without static checking.

## 修正方針

Make WASIX TUI import and use the typed std/stdio/ansi API. Replace raw i32 color signatures with AnsiColor/AnsiTextStyle helpers, have style_text accept AnsiTextStyle, and add source policy plus stdout doctests that prevent numeric color helpers from returning.

## 検証

Run focused features_tui tests, stdio ansi policy, wasix tui source policy, and issue index checks.

## 対応

- `std/stdio/ansi.nepl` の `AnsiTextStyle` に `foreground` / `background` / `weight` / `decoration` を持たせ、背景色も typed style の一部として扱えるようにした。
- `ansi_background_color_code`、`ansi_text_style_code`、`ansi_background_color_style`、`ansi_color_pair_style`、`ansi_bold_color_pair_style` を追加し、foreground/background の code 生成を wildcard なしの `match` に集約した。
- `platforms/wasix/tui.nepl` の `set_fg_color` / `set_bg_color` は `AnsiColor`、`style_text` / `line_box_styled` は `AnsiTextStyle` を受け取る API に変更した。
- TUI の `get_tty_state` sentinel API を `get_tty_state_result` に置き換え、成功 pointer と失敗 errno を `Result<i32,i32>` で分けた。これにより、解放済み pointer と有効 pointer が同じ raw `i32` として戻る経路を閉じた。
- `tests/stdlib/features_tui.n.md` に typed color stdout 回帰を追加し、既存 box helper 回帰も `AnsiTextStyle` 呼び出しへ更新した。
- `nodesrc/test_stdlib_stdio_ansi_boundary.js` と `nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js` を拡張し、背景色 code の網羅的 match、shared style code helper、TUI raw `i32` color API と raw pointer sentinel の再導入禁止を固定した。

## 検証結果

- `node nodesrc/test_stdlib_stdio_ansi_boundary.js`: passed
- `node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_stdio_debug_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/ansi.nepl --no-tree -o tmp/tui-typed-color-stdio-ansi-after-result.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/std/stdio/debug.nepl --no-tree -o tmp/tui-typed-color-stdio-debug.json -j 1 --dist web/dist`: total=8, passed=8
- `node nodesrc/tests.js -i tests/stdlib/stdout.n.md --no-tree -o tmp/tui-typed-color-stdout-after-result.json -j 1 --dist web/dist`: total=7, passed=7
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-typed-color-features-after-result.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-typed-color-features-with-facade.json -j 1 --dist web/dist`: total=6, passed=6
