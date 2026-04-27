---
id: ISS-20260427T000313614Z-ALLOC-STRING-NEEDS-OWNERSHIP-SAFE-UT-53AE8496
title: "alloc/string needs ownership-safe UTF-8 reimplementation"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/tests/string.n.md, tests/stdlib/string.n.md, stdlib/std/fs.nepl"
---

# ISS-20260427T000313614Z-ALLOC-STRING-NEEDS-OWNERSHIP-SAFE-UT-53AE8496: alloc/string needs ownership-safe UTF-8 reimplementation

## 概要

alloc/string still mixes raw region layout, unwrap_ok/unwrap/unreachable paths, byte-based constructors, and older compiler-workaround temporaries, so str invariants and allocation failures are not represented consistently.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/tests/string.n.md, tests/stdlib/string.n.md, stdlib/std/fs.nepl`

## 根拠

- `stdlib/alloc/string.nepl` の実装本体に `unwrap_ok` / `unwrap` / `#intrinsic "unreachable"` が残り、allocation failure や owned string header store が trap へ寄っていた。
- `concat` / `str_slice` / `str_split` / `StringBuilder` は allocation-bearing だが、Result-returning の self-host 向け入口がなかった。
- `ByteBuf -> str` の詰め直しが `alloc/io` 側にも重複し、string layout の責務が分散していた。

## 問題

alloc/string still mixes raw region layout, unwrap_ok/unwrap/unreachable paths, byte-based constructors, and older compiler-workaround temporaries, so str invariants and allocation failures are not represented consistently.

## 影響

Self-host lexer, parser, diagnostics, module paths, HTML/NM generation, and file loading all depend on trustworthy UTF-8 strings and predictable Result-returning string operations.

## 修正方針

Rework the string core around explicit owned raw operations and checked UTF-8 construction, remove compiler-workaround intermediate variables where the fixed compiler allows direct expressions, and expand regression coverage for allocation, slicing, search, formatting, and invalid UTF-8 boundaries.

## 対応結果

- `string_finish_base` を owned string header への raw store に変更し、checked `store_i32` + unreachable 経路を削除した。
- `string_from_mem_unchecked_result` と `string_from_utf8_mem_result` を追加し、raw byte 複製と UTF-8 checked construction を `alloc/string` の中核 API として分離した。
- `StringUtf8LeadKind` enum と `match` により UTF-8 sequence 長の網羅分岐を明示した。
- `concat_result` / `str_slice_result` / `str_split_result` / `string_builder_new_result` / `sb_append_result` / `sb_append_i32_result` / `sb_build_result` / `from_f64_result` を追加し、self-host から allocation failure を Result として扱える入口を整えた。
- `from_u128_radix` と `from_f64_result` の scratch buffer を raw owned storage 管理に整理し、`unwrap` / `unreachable` を削除した。
- `alloc/io.io_bytebuf_to_str_result` は `string_from_mem_unchecked_result` へ委譲し、string layout の詰め直しを `alloc/string` に集約した。binary 用の unchecked API は互換入口として残し、source text では既存の checked API を使う設計を維持した。
- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` を追加し、CI source policy と `doc/testing.md` に登録した。

## 検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/alloc/io.nepl --no-tree -o tmp/string-refactor-docs-io.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md -i tests/stdlib/text_utf8.n.md -i tests/stdlib/bytebuf_result.n.md -i tests/stdlib/fs.n.md --no-tree -o tmp/string-refactor-focused.json -j 1`: 42/42 passed
