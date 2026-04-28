---
id: ISS-20260428T004640143Z-STDLIB-NEEDS-CHAR-API-AND-STRING-INT-547D97E5
title: "stdlib needs char API and string integration plan"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "doc/neplg2/char_stdlib_integration_plan.md, nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/tests/char.rs, stdlib/core/cast.nepl, stdlib/core/char.nepl, stdlib/alloc/string.nepl, stdlib/std/text.nepl, stdlib/alloc/io.nepl, tests/compiler/char_cast.n.md, tests/stdlib/string_char.n.md, tests/stdlib/text_utf8.n.md"
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

## 解決

- `core/cast` に `char -> i32` と `i32 -> char` の明示変換を追加し、WASM / LLVM codegen と typecheck の intrinsic 対応を追加した。
- `stdlib/core/char.nepl` を追加し、Unicode scalar 検証、`CharUtf8Step`、ASCII 分類、UTF-8 encode 用 byte helper を `alloc` 非依存で提供した。
- `alloc/string` に `str_byte_len`、`str_next_char_result`、`str_char_count`、`str_char_at_result`、`str_slice_chars_result`、`str_starts_with_char`、`str_contains_char`、`sb_append_char(_result)`、`sb_append_ascii(_result)` を追加した。
- `alloc/io` と `std/text` に `char` を UTF-8 `ByteBuf` へ encode する API と raw byte から 1 char を decode する API を追加した。
- 旧 tuple 型は使わず、decode step は `CharUtf8Step { value, next }` で返す仕様として計画書にも反映した。

## 検証

Add doctests for core/char, tests/stdlib/string_char.n.md, text UTF-8 encode/decode tests, and static regression tests for classifier functions. Run focused string/text/json/nm/html/cliarg/byte_builder/selfhost lexer tests and issue checks.

- `cargo check`: pass
- `cargo test -p nepl-core --test char`: 12/12 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/char_cast.n.md --no-tree -o tmp/char-cast-after-rebase.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/core/char.nepl --no-tree -o tmp/core-char-docs-after-rebase.json -j 1`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/string-char-after-rebase.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text-utf8-char-after-rebase.json -j 1`: 9/9 passed
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-char-docs-after-rebase.json -j 1`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/alloc-string-char-docs-after-rebase.json -j 1`: 6/6 passed
