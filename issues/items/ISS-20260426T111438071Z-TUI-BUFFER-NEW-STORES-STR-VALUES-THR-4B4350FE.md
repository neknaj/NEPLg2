---
id: ISS-20260426T111438071Z-TUI-BUFFER-NEW-STORES-STR-VALUES-THR-4B4350FE
title: "TUI buffer_new stores str values through i32 store"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md"
---

# ISS-20260426T111438071Z-TUI-BUFFER-NEW-STORES-STR-VALUES-THR-4B4350FE: TUI buffer_new stores str values through i32 store

## 概要

`features/tui` facade doctest が実行前に compile fail する。原因は `buffer_new` が `str` 行スロットを初期化するとき、`store_i32 add curr off ""` / `store_i32 add prev off ""` で `str` 値を i32 store に渡していること。

## 対象

- `stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-narrow-baseline.json -j 1` が `/stdlib/platforms/wasix/tui.nepl:707` と `:708` の `store_i32` で `D3006 no matching overload found` を返した。

## 問題

`buffer_new` の `curr` / `prev` 配列は `buffer_set_line` でも `load<str>` / `store<str>` される `str` スロットである。しかし初期化だけ `store_i32` を使っていたため、`str` と raw `i32` handle の分離後に TUI module 全体が型検査で落ちる。

## 影響

Any import of features/tui compiles the whole platforms/wasix/tui module, so pure helper doctests and self-host TUI users cannot compile even when they do not call buffer_new.

## 修正方針

Use typed store<str> for the curr/prev line slots and add a regression doctest that constructs a buffer through the facade.

## 検証

node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-buffer-str-store.json -j 1

## 対応

- `buffer_new` の空文字列初期化を `store<str>` へ変更し、同じ配列に対する `buffer_set_line` の store 型と揃えた。
- `tests/stdlib/features_tui.n.md` に `buffer_new` / `buffer_set_line` / `buffer_free` の最小経路を追加し、facade import だけで TUI module が compile fail しないことを固定した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-buffer-str-store.json -j 1`: `total=3`, `passed=3`
- `node nodesrc/tests.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md --no-tree -o tmp/tui-buffer-focused-files.json -j 1`: `total=4`, `passed=4`
- `node nodesrc/issues.js index` / `node nodesrc/issues.js check`: pass
- `cargo fmt --all --check`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tui-buffer-str-store.json`: `13/13 passed`
- `git diff --check`: pass（issue index の CRLF warning のみ）
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/tui-buffer-stdlib-full.json -j 4`: timeout after 304s, `partial=true`, `completed_results=0`。今回の focused tests は通過しており、full stdlib は別 node process と重なったため再試行対象。
