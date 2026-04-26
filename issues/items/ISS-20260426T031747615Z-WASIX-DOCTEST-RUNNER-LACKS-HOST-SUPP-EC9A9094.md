---
id: ISS-20260426T031747615Z-WASIX-DOCTEST-RUNNER-LACKS-HOST-SUPP-EC9A9094
title: "wasix doctest runner lacks host support for tty_get and tty_set imports"
area: cli
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/run_test.js, tests/stdlib/features_tui.n.md"
---

# ISS-20260426T031747615Z-WASIX-DOCTEST-RUNNER-LACKS-HOST-SUPP-EC9A9094: wasix doctest runner lacks host support for tty_get and tty_set imports

## 概要

Wasmer 1.x の mount 引数互換を直した後、tests/stdlib/features_tui.n.md は run phase で wasix_32v1.tty_get unknown import に進む。TUI pure helper しか使わないケースでも platforms/wasix/tui の #extern が wasm import として要求されるため、runner が TTY host import を提供しない環境では実行できない。

## 対象

- `nodesrc/run_test.js, tests/stdlib/features_tui.n.md`

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

## 対応結果

`nodesrc/run_test.js` の WASIX 実行を、まず従来どおり Wasmer で試し、Wasmer が `wasix_32v1.tty_get` / `tty_set` の unknown import で落ちた場合だけ Node WASI 実行へ fallback する形にした。
fallback 側は `wasix_32v1.tty_get` / `tty_set` を host import として提供し、どちらも失敗 errno を返す。
これにより `stdlib/platforms/wasix/tui.nepl` 側の既存仕様どおり、TTY が取れない環境では `get_terminal_size` が `TerminalSize 0 0` を返す。

TUI module 分割は今後の設計余地として残るが、今回の根本原因は doctest runner が known optional host import を提供できず実行前 link で落ちることだったため、runner fallback で修正した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
- `node --check nodesrc/run_test.js`
- WASIX smoke を `nodesrc/run_test.js` へ直接渡した実行が `ok=true` で完了。
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/wasix-tty-tests-stdlib.json -j 4` (`total=202`, `passed=201`, `failed=1`)。残り1件は既存 `ISS-20260426T023700894Z-TRAITS-TEXT-DOCTEST-RUNTIME-RETURN-M-D1631318`。
