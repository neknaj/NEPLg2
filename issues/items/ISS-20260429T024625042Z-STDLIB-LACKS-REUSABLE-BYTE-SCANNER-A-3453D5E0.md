---
id: ISS-20260429T024625042Z-STDLIB-LACKS-REUSABLE-BYTE-SCANNER-A-3453D5E0
title: "stdlib lacks reusable byte scanner and ASCII classification helpers"
area: stdlib
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl"
---

# ISS-20260429T024625042Z-STDLIB-LACKS-REUSABLE-BYTE-SCANNER-A-3453D5E0: stdlib lacks reusable byte scanner and ASCII classification helpers

## 概要

Audit found repeated scanner-local helpers for line_end, find_byte, skip_space, ASCII digit/alpha/space classification, and byte marker detection across self-host lexer/import parsing and nm parser/html generation.

## 対象

- `stdlib/alloc/string.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl`

## 根拠

- `stdlib/neplg2/core/syntax/lexer.nepl` は `lex_is_digit` / `lex_is_alpha` / `lex_is_ident_start` / `lex_skip_horizontal` / `lex_line_end` を scanner-local に持つ。
- `stdlib/neplg2/core/module/import_spec.nepl` は `selfhost_import_find_byte` / `selfhost_import_is_space` / `selfhost_import_skip_space` / `selfhost_import_word_end` を持ち、lexer と空白分類・byte range scan が重複している。
- `stdlib/nm/parser.nepl` と `stdlib/nm/html_gen.nepl` は `nm_line_end` / `nm_find_byte` 相当の byte loop と inline marker 判定をそれぞれ持ち、同じ Gloss/NM domain 内でも scan helper が再利用されていない。
- `alloc/string.nepl` には `find(str, str)` と `str_starts_with_at` はあるが、byte range find、line end、ASCII classification、prefix consume のような parser 向けの薄い helper がまだない。

## 問題

Audit found repeated scanner-local helpers for line_end, find_byte, skip_space, ASCII digit/alpha/space classification, and byte marker detection across self-host lexer/import parsing and nm parser/html generation.

## 影響

Parser-like modules keep reimplementing byte loops with slightly different boundary rules, which makes future self-host and nm work harder to review and increases the chance of off-by-one or CRLF/whitespace inconsistencies.

## 修正方針

Design a small stdlib scanner layer: byte range find, line end/next line helpers, ASCII classification predicates, and optionally prefix-consuming helpers that return the next offset. Refactor nm and self-host scanner modules to use those helpers after the string prefix-at API lands.

## 対応

`alloc/string.nepl` に parser 向けの byte scanner helper を追加した。追加した API は、`str_find_byte_range`、`str_line_end`、`str_next_line_pos`、`str_trim_suffix_cr`、`str_skip_inline_space_range`、`str_word_end_inline_space_range`、および ASCII byte 分類 `str_byte_is_ascii_*` である。

`stdlib/neplg2/core/module/import_spec.nepl` から scanner-local な `selfhost_import_find_byte` / `selfhost_import_is_space` / `selfhost_import_skip_space` / `selfhost_import_word_end` を削除し、`alloc/string` の helper へ置き換えた。

`stdlib/nm/parser.nepl` と `stdlib/nm/html_gen.nepl` から `nm_line_end` / `nm_next_line_pos` / `trim_cr` / `nm_find_byte` への依存を取り除き、共通 helper を使う形にした。`nodesrc/test_stdlib_byte_scanner_helpers_boundary.js` を追加し、同じ local helper が戻らないことを固定した。

## 検証

- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\stdlib-byte-scanner-string-final2.json -j 1`: total=8 passed=8
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed（CRLF warning のみ）
- `node nodesrc/tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\byte-scanner-import-spec.json -j 1`: known D3100 で failed。`ISS-20260429T024412130Z-RESOURCE-OWNER-GATE-REPORTS-D3100-IN-7A19FECC` / `ISS-20260429T021254285Z-RESOURCE-OWNER-GATE-LEAKS-OBLIGATION-8F3BD354` で追跡。
- `node nodesrc/tests.js -i tests\stdlib\nm.n.md --no-tree -o tmp\byte-scanner-nm.json -j 1`: known D3100 で failed。`ISS-20260429T030655089Z-RESOURCE-OWNER-GATE-REGRESSES-NM-DIR-98E651E0` で追跡。
