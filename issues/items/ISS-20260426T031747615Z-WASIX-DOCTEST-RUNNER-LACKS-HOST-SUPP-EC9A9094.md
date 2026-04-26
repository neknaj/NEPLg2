---
id: ISS-20260426T031747615Z-WASIX-DOCTEST-RUNNER-LACKS-HOST-SUPP-EC9A9094
title: "wasix doctest runner lacks host support for tty_get and tty_set imports"
area: cli
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/run_test.js, stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md"
---

# ISS-20260426T031747615Z-WASIX-DOCTEST-RUNNER-LACKS-HOST-SUPP-EC9A9094: wasix doctest runner lacks host support for tty_get and tty_set imports

## 概要

Wasmer 1.x の mount 引数互換を直した後、tests/stdlib/features_tui.n.md は run phase で wasix_32v1.tty_get unknown import に進む。TUI pure helper しか使わないケースでも platforms/wasix/tui の #extern が wasm import として要求されるため、runner が TTY host import を提供しない環境では実行できない。

## 対象

- `nodesrc/run_test.js, stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md`

## 根拠

- `ISS-20260426T030615554Z-WASIX-DOCTEST-RUNNER-USES-WASMER-VOL-8527FD91` の修正後、`node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1` は mount option error ではなく `Error while importing "wasix_32v1"."tty_get": unknown import` で失敗する。
- `stdlib/platforms/wasix/tui.nepl` は file 先頭で `#extern "wasix_32v1" "tty_get"` と `tty_set` を宣言している。
- `features_tui_facade_reexports_text_helpers` は `line_pad_to_cols` / `repeat_text` だけを使うが、module import 時点で TTY extern が wasm import に含まれるため、pure helper の検証でも host TTY import が必要になる。

## 問題

Wasmer 1.x の mount 引数互換を直した後、tests/stdlib/features_tui.n.md は run phase で wasix_32v1.tty_get unknown import に進む。TUI pure helper しか使わないケースでも platforms/wasix/tui の #extern が wasm import として要求されるため、runner が TTY host import を提供しない環境では実行できない。

## 影響

features/tui の facade 回帰が compile では確認できても run で green にできず、TUI helper と get_terminal_size の実行時契約を CI/ローカルで検証できない。

## 修正方針

runner 側で tty_get/tty_set の host fallback を提供するか、TUI module を pure helper と host TTY binding に分割して、pure helper の利用時に不要な extern import を要求しないようにする。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
