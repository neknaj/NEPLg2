---
id: ISS-20260428T004640143Z-STDLIB-NEEDS-CHAR-API-AND-STRING-INT-547D97E5
title: "stdlib needs char API and string integration plan"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "doc/neplg2/char_stdlib_integration_plan.md, stdlib/core/char.nepl, stdlib/alloc/string.nepl, stdlib/std/text.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md"
---

# ISS-20260428T004640143Z-STDLIB-NEEDS-CHAR-API-AND-STRING-INT-547D97E5: stdlib needs char API and string integration plan

## 概要

The core char literal issue defines the language feature, and the existing stdlib migration issue tracks replacing magic character codes. However, stdlib still needs a coherent char API plan: conversions, ASCII classification, UTF-8 encode/decode, str char iteration, StringBuilder/ByteBuilder integration, and naming rules that keep byte length distinct from char count.

## 対象

- `doc/neplg2/char_stdlib_integration_plan.md, stdlib/core/char.nepl, stdlib/alloc/string.nepl, stdlib/std/text.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md`

## 関連ドキュメント

- [NEPLg2 stdlib char 整備計画](../../doc/neplg2/char_stdlib_integration_plan.md)

## 根拠

- `stdlib/alloc/string.nepl` は UTF-8 validation と boundary check を持つが、public API は byte index / byte length が中心で、char count / char at / char slice API がない。
- `stdlib/std/text.nepl` は external bytes を UTF-8 `str` へ変換するが、char encode / decode API としては公開されていない。
- `StringBuilder` / `ByteBuilder` は `str` 片や raw byte を追加できるが、`char` を UTF-8 encode して追加する API を持たない。
- `char` 導入後に `len` の意味を char count へ変えると既存 byte-oriented stdlib と WASI I/O を壊すため、byte API と char API の命名分離が必要である。

## 問題

The core char literal issue defines the language feature, and the existing stdlib migration issue tracks replacing magic character codes. However, stdlib still needs a coherent char API plan: conversions, ASCII classification, UTF-8 encode/decode, str char iteration, StringBuilder/ByteBuilder integration, and naming rules that keep byte length distinct from char count.

## 影響

If char lands without stdlib integration, string and parser code will either keep using decimal byte codes or introduce incompatible ad hoc helpers. That would blur byte index vs char index, risk changing the meaning of len/str_slice, and make self-host lexer/parser code inconsistent.

## 修正方針

Implement doc/neplg2/char_stdlib_integration_plan.md. Add stdlib/core/char.nepl with conversion and ASCII classifier APIs; add string char APIs such as str_byte_len, str_char_count, str_char_at_result, str_next_char_result, str_slice_chars_result; add StringBuilder and ByteBuilder char append helpers; share UTF-8 decode/encode logic with std/text; then migrate existing classifiers and lexer/parser punctuation to char literals where the value denotes a character.

## 検証

Add doctests for core/char, tests/stdlib/string_char.n.md, text UTF-8 encode/decode tests, and static regression tests for classifier functions. Run focused string/text/json/nm/html/cliarg/byte_builder/selfhost lexer tests and issue checks.
