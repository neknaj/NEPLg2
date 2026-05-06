---
id: ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044
title: "String float and char builder owner chains fail strict ResourceIR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/alloc/string/float.nepl, stdlib/alloc/string/builder.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md"
---

# ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044: String float and char builder owner chains fail strict ResourceIR

## 概要

Focused string verification after alloc/string facade split reaches ResourceIR and reports resource.owner.use_after_move/reserved in from_f64_append_fraction_result, from_f64_build_fixed_result, string_char.n.md char slice checks, and ByteBuilder finish chains. The failures occur on Result-returning builder owners that should transfer exactly once through Ok arms.

## 対象

- `stdlib/alloc/string/float.nepl, stdlib/alloc/string/builder.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md`

## 根拠

- 未記入

## 問題

Focused string verification after alloc/string facade split reaches ResourceIR and reports resource.owner.use_after_move/reserved in from_f64_append_fraction_result, from_f64_build_fixed_result, string_char.n.md char slice checks, and ByteBuilder finish chains. The failures occur on Result-returning builder owners that should transfer exactly once through Ok arms.

## 影響

String numeric formatting and char/byte builder tests cannot be used as a clean regression signal under mandatory memory-safety checking. This can hide real builder leaks or push developers toward weakening ResourceIR diagnostics.

## 修正方針

Trace the builder owner summaries and call-site Result arm refinement without weakening ResourceIR. If stdlib code is relying on ambiguous owner flow, rewrite the builder chains so each owner is consumed or freed in a statically visible path and add focused regression tests for from_f64 and char builders.

## 検証

Run focused string float and string_char doctests, source policy string owner checks, and ResourceIR owner regressions.

## 2026-05-07 Agent 2 float formatter 部分進捗

`stdlib/alloc/string/float.nepl::doctest#1` の `from_f64_build_fixed_result` は、`StringBuilder` の `Result<StringBuilder, str>` owner chain をまたいで小数部を追加していたため、strict Resource IR で `sb2` の backing pointer が moved と判定されていた。

修正:

- 固定小数 formatter は最終出力 byte 数を事前に持っているため、growable `StringBuilder` を使わず、`string_alloc_region` で出力 `RegionToken` を 1 回だけ確保する構造へ変更した。
- 符号、整数部、小数点、小数 digit を同じ出力領域へ順に書き、最後に `string_finish` で `str` へ確定する。
- 小数 digit の有限分岐は `match trim` で 0..6 と `_` を列挙し、trim 値が検査から外れた場合は出力 region を解放して `Err` を返す。
- `alloc/string/float.nepl` は `alloc/string/integer.nepl` や `alloc/string/concat.nepl` と同じく string storage raw write boundary になったため、loader の configured raw-memory boundary path に追加した。

検証:

- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_byte_and_scanner_boundaries_as_raw_memory_boundary -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/alloc/string/float.nepl --no-tree -o tmp/string-float-owner-direct-region-after-trunk.json -j 1`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md -i stdlib/alloc/string/float.nepl -i stdlib/alloc/string/builder.nepl -i stdlib/alloc/io.nepl --no-tree -o tmp/string-builder-owner-after-float-direct-region.json -j 1`: total=5, passed=3, failed=2

残件:

- `tests/stdlib/string_char.n.md::doctest#3` は `byte_builder_push_char_utf8` で multi-byte char を追加した後の `byte_builder_finish b2` が `resource.owner.use_after_move` になる。`alloc/io.nepl` の stdlib-only 実験として UTF-8 tail helper の `match` 化、reserve 1 回 + direct store 化、raw store 化を試したが、`Result<ByteBuilder, StdErrorKind>` の multi-step owner summary は安定しなかったため未採用。
- `tests/stdlib/string_char.n.md::doctest#1` は `str_slice_chars_result s 1 3` の成功後に同じ `s` を読むと `resource.owner.reserved` になる。これは builder chain ではなく、`str_slice_result` / `string_from_mem_unchecked_result` が source `str` から新しい `str` を複製した後の Resource IR returned raw header / source view summary の問題として扱う。
