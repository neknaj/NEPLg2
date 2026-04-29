---
id: ISS-20260429T024625042Z-STDLIB-LACKS-REUSABLE-BYTE-SCANNER-A-3453D5E0
title: "stdlib lacks reusable byte scanner and ASCII classification helpers"
area: stdlib
status: open
resolved: false
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

## 検証

Add focused tests for byte range find, line end, CRLF trimming, ASCII classification, then run nm, self-host lexer, and import spec fixtures.
