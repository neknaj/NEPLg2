---
id: ISS-20260426T023638576Z-FEATURES-TUI-DOCTEST-HELPER-988788D1
title: "features_tui doctest が未定義 helper 参照で失敗する"
area: stdlib
status: open
resolved: false
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-26
target: "tests/stdlib/features_tui.n.md, stdlib/std/features/tui.nepl"
---

# ISS-20260426T023638576Z-FEATURES-TUI-DOCTEST-HELPER-988788D1: features_tui doctest が未定義 helper 参照で失敗する

## 概要

tests/stdlib/features_tui.n.md が tui::line_pad_to_cols、tui::repeat_text、tui::get_terminal_size の D3001 undefined identifier で失敗する。

## 対象

- `tests/stdlib/features_tui.n.md, stdlib/std/features/tui.nepl`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/rv-stdlib-018-final-tests-stdlib-crlf.json -j 4` で `tests/stdlib/features_tui.n.md::doctest#1` と `doctest#2` が compile failure になった。
- `doctest#1` は `tui::line_pad_to_cols` と `tui::repeat_text` が `D3001 undefined identifier` になり、後続で `D3016` が連鎖する。
- `doctest#2` は `tui::get_terminal_size` が `D3001 undefined identifier` になり、`cols` / `rows` の取得と条件式で `D3016` / `D3039` が連鎖する。

## 問題

tests/stdlib/features_tui.n.md が tui::line_pad_to_cols、tui::repeat_text、tui::get_terminal_size の D3001 undefined identifier で失敗する。

## 影響

tests/stdlib 全体の green 化を阻害し、TUI feature の公開 API と fixture のどちらが正しいか検証できない。

## 修正方針

stdlib/std/features/tui.nepl の公開 API と doctest の意図を照合し、仕様上必要な helper は実装し、既存 API へ統合済みなら doctest を現行名へ更新する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-issue.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/features-tui-tests-stdlib.json -j 4`
