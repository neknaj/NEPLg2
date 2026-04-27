---
id: ISS-20260427T060739844Z-WASIX-TUI-TEXT-WRAP-LINES-VEC-ALLOCA-405DA6BF
title: "WASIX TUI text_wrap_lines が Vec allocation failure を unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js"
---

# ISS-20260427T060739844Z-WASIX-TUI-TEXT-WRAP-LINES-VEC-ALLOCA-405DA6BF: WASIX TUI text_wrap_lines が Vec allocation failure を unwrap_ok で trap する

## 概要

text_wrap_lines allocates and appends Vec<str> lines with unwrap_ok new/push, so wrapping long text can trap on allocation or grow failure instead of returning a safe empty/partial result.

## 対象

- `stdlib/platforms/wasix/tui.nepl, tests/stdlib/features_tui.n.md, nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`

## 根拠

- `text_wrap_lines` は `out` を `unwrap_ok new<str>` で生成していた。
- 改行、折り返し、tail の各行 push を `unwrap_ok push<str>` で行っていた。

## 問題

text_wrap_lines allocates and appends Vec<str> lines with unwrap_ok new/push, so wrapping long text can trap on allocation or grow failure instead of returning a safe empty/partial result.

## 影響

TUI helpers are used by stdlib feature tests and future self-host CLI UIs. Memory pressure in rendering can abort the program and keeps RV-STDLIB-010 unsafe helper debt in normal implementation code.

## 修正方針

Qualify Vec operations through a vec alias, replace unwrap_ok with explicit Result matches, stop line accumulation on push failure, and return an empty Vec sentinel on allocation failure.

## 検証

- `node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/wasix-tui-wrap-allocation-focused.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-wasix-tui-wrap-allocation.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass (CRLF conversion warning only)

## 解決内容

- `TuiStrPushRes` / `tui_empty_str_vec` / `tui_push_str` を追加し、折り返し行 `Vec<str>` の push failure を `ok=false` と空 Vec sentinel に変換した。
- `text_wrap_lines` の `new<str>` / `push<str>` から implementation `unwrap_ok` を除去した。
- line scan は `failed=true` で停止し、allocation failure 時は consumed owner を再利用しない空 Vec sentinel を返すようにした。
- `text_wrap_lines` 内部の Vec 操作を `v::new` / `v::push` / `v::Vec` に限定した。
- `nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js` を追加し、CI/source policy と `doc/testing.md` に登録した。
