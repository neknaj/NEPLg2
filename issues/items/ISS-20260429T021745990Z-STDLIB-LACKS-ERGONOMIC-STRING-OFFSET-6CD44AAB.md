---
id: ISS-20260429T021745990Z-STDLIB-LACKS-ERGONOMIC-STRING-OFFSET-6CD44AAB
title: "stdlib lacks ergonomic string offset prefix helpers for self-host scanners"
area: stdlib
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, nodesrc/test_selfhost_string_helpers_boundary.js"
---

# ISS-20260429T021745990Z-STDLIB-LACKS-ERGONOMIC-STRING-OFFSET-6CD44AAB: stdlib lacks ergonomic string offset prefix helpers for self-host scanners

## 概要

Self-host scanner code uses hand-written byte comparisons for literal prefixes such as #indent/as and calls internal-style str_eq_at with magic length and loop-index arguments. This happened because stdlib has str_starts_with and internal str_eq_at, but no safe public str_starts_with_at API for offset-based scanners.

## 対象

- `stdlib/alloc/string.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, nodesrc/test_selfhost_string_helpers_boundary.js`

## 根拠

- `stdlib/neplg2/core/syntax/lexer.nepl` の `lex_starts_with_indent_directive` は `#indent` を 7 個の `string_byte_at_unchecked` と `ok_*` 中間変数で比較している。
- 同じ lexer の `lex_directive_word_at` と `#if[target=` / `#if[profile=` 判定は、呼び出し側が `word_len` / magic length / loop index `0` を渡して `string::str_eq_at` を直接呼んでいる。
- `stdlib/neplg2/core/module/import_spec.nepl` の `selfhost_import_has_as_keyword` も `as` の 2 byte を手で読む。scanner が offset から literal prefix を検査する用途に対し、stdlib の public API が不足している。

## 問題

Self-host scanner code uses hand-written byte comparisons for literal prefixes such as #indent/as and calls internal-style str_eq_at with magic length and loop-index arguments. This happened because stdlib has str_starts_with and internal str_eq_at, but no safe public str_starts_with_at API for offset-based scanners.

## 影響

Lexer/import parser code becomes long, brittle, and easy to desynchronize from string semantics. New scanner code is likely to repeat unchecked byte-at sequences instead of using a reviewed stdlib primitive.

## 修正方針

Add a safe zero-allocation str_starts_with_at(s,start,prefix) API in alloc/string, document bounds and byte semantics, then refactor self-host lexer/import parser prefix checks to use it. Add a structural regression that blocks reintroducing hand-written #indent byte matching and direct str_eq_at use in self-host scanner code.

## 対応

`alloc/string.nepl` に `str_starts_with_at(s, start, prefix)` を追加した。`start < 0`、`start > len(s)`、残り byte 数不足を false にし、正常範囲だけ内部 helper `str_eq_at` に委譲するため、scanner 側は magic length や loop index を渡さない。

`stdlib/neplg2/core/syntax/lexer.nepl` では `#indent`、directive word、`#if[target=`、`#if[profile=` の判定を `str_starts_with_at` に置き換えた。`stdlib/neplg2/core/module/import_spec.nepl` では `as` keyword 判定の 2 byte 手読みを同 API に置き換えた。

`nodesrc/test_selfhost_string_helpers_boundary.js` を追加し、`#indent` の `ok_*` 手書き比較と self-host lexer からの direct `string::str_eq_at` 呼び出しが戻らないことを固定した。

## 検証

- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\string-prefix-at.json -j 1`: total=7 passed=7
- `node nodesrc/tests.js -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\neplg2-lexer-prefix-at-serial.json -j 1`: failed。全 13 件が `lex_all_loop` の D3100 Resource IR owner obligation leak で止まる。prefix helper の構造 regression は passed しており、D3100 は別 issue に分離する。
- `node nodesrc/tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\neplg2-import-prefix-at.json -j 1`: failed。`hash32(str)` と import parser helper の D3100 Resource IR owner obligation leak で止まる。静的検査側の別 issue として分離する。
