---
id: ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB
title: "stdlib and selfhost should replace character code magic numbers with char literals"
area: stdlib
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/std/stdio.nepl, stdlib/std/env/cliarg.nepl, stdlib/alloc/io.nepl, stdlib/platforms/wasix/tui.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib, tests/compiler/char_literals.n.md"
---

# ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB: stdlib and selfhost should replace character code magic numbers with char literals

## 概要

Many existing stdlib and selfhost modules classify ASCII/control bytes with decimal literals even when the branch means a concrete character. Examples include JSON/nm/html escape classifiers, selfhost lexer punctuation, string escape/numeric parsing, stdio sign handling, cliarg C-string tests, byte builder magic bytes, and WASIX TUI escape handling. Once char literals exist, leaving these as numbers will keep the original readability problem and make the new feature underused.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, stdlib/std/stdio.nepl, stdlib/std/env/cliarg.nepl, stdlib/alloc/io.nepl, stdlib/platforms/wasix/tui.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib, tests/compiler/char_literals.n.md`

## 関連ドキュメント

- [NEPLg2 stdlib char 整備計画](../../doc/neplg2/char_stdlib_integration_plan.md)

## 根拠

- `stdlib/nm/parser.nepl` は JSON escape classifier で `match ch:` に `92` / `34` / `10` / `13` / `9` / `8` / `12` を使っている。
- `stdlib/nm/html_gen.nepl` は HTML escape classifier で `38` / `60` / `62` / `34` / `39` を使っている。
- `stdlib/neplg2/core/syntax/lexer.nepl` は punctuation / string / comment 周辺の判定で character code を直接扱う。
- `stdlib/std/env/cliarg.nepl` の doctest は C-string bytes を `110` / `101` / `112` / `0` のように書いている。
- `tests/stdlib/byte_builder.n.md` は magic bytes のうち text 部分も `97` / `115` / `109` などの decimal code で検査している。

## 問題

Many existing stdlib and selfhost modules classify ASCII/control bytes with decimal literals even when the branch means a concrete character. Examples include JSON/nm/html escape classifiers, selfhost lexer punctuation, string escape/numeric parsing, stdio sign handling, cliarg C-string tests, byte builder magic bytes, and WASIX TUI escape handling. Once char literals exist, leaving these as numbers will keep the original readability problem and make the new feature underused.

## 影響

Reviewers cannot tell whether a number is a byte value, a length, an offset, a tag, or a character. This is especially harmful in lexer/parser/string code where values like 10, 13, 34, 39, 60, 62, 92, 123, and 125 encode syntax. It also makes match-based finite dispatch less self-documenting.

## 修正方針

After core char support lands, audit stdlib and selfhost code and replace character-code literals with char literals where the value denotes a character. Prioritize escape classifiers and lexer/parser punctuation: use '\n', '\r', '\t', '\\', '\'', '"', '&', '<', '>', '/', '[', ']', '{', '}', '$', '-' instead of decimal codes. Keep numeric literals for sizes, offsets, enum tags, non-text binary formats, and non-printable binary constants where char would mislead. Add focused tests and a static search/regression rule for known character-code comparisons in classifier functions.

## 検証

Run targeted tests for string, json, nm, html_gen, stdio, cliarg, byte_builder, WASIX TUI if available, and stdlib/neplg2 lexer. Add a static test that key classifier functions use char literal match arms or comparisons rather than decimal character codes.
