---
id: ISS-20260518T072300172Z-ALLOC-STRING-SCANNER-EXPOSES-UNCHECK-BE608F25
title: "alloc string scanner exposes unchecked byte reader"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/scanner.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_byte_scanner_helpers_boundary.js"
---

# ISS-20260518T072300172Z-ALLOC-STRING-SCANNER-EXPOSES-UNCHECK-BE608F25: alloc string scanner exposes unchecked byte reader

## 概要

alloc/string/scanner publishes scanner_string_byte_at_unchecked even though the function is only used inside scanner.nepl after local range normalization. Direct import users can call the unchecked reader with arbitrary indices instead of going through scanner range helpers.

## 対象

- `stdlib/alloc/string/scanner.nepl`
- `tests/stdlib/memory_safety.n.md`
- `nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`

## 根拠

- `rg` found `scanner_string_byte_at_unchecked` only in `stdlib/alloc/string/scanner.nepl` and source policy text.
- Public scanner APIs already normalize ranges in `str_find_byte_range`, `str_line_end`, `str_next_line_pos`, `str_skip_inline_space_range`, and `str_word_end_inline_space_range` before calling the unchecked reader.
- No external stdlib caller needs the unchecked reader directly.

## 問題

alloc/string/scanner publishes scanner_string_byte_at_unchecked even though the function is only used inside scanner.nepl after local range normalization. Direct import users can call the unchecked reader with arbitrary indices instead of going through scanner range helpers.

## 影響

The scanner module leaks a raw string-layout read helper into the public API. This expands raw-memory-backed authority beyond the module-local proof boundary and conflicts with Stage 6 public/internal separation.

## 修正方針

Make scanner_string_byte_at_unchecked private, keep public scanner APIs on bounded range helpers, and add source policy plus memory_safety compile_fail coverage for direct import.

## 検証

Run scanner source policy and focused memory_safety/string scanner doctests.

## 修正内容

- `scanner_string_byte_at_unchecked` を private helper に変更した。
- public API は range normalization を行う scanner helper と ASCII byte classification helper に限定した。
- `nodesrc/test_stdlib_byte_scanner_helpers_boundary.js` に private boundary policy を追加した。
- `tests/stdlib/memory_safety.n.md` に direct import から unchecked reader が見えない compile_fail regression を追加した。
