---
id: ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044
title: "String float and char builder owner chains fail strict ResourceIR"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/owner_*.rs, stdlib/alloc/string/float.nepl, stdlib/alloc/string/slice.nepl, stdlib/alloc/string/char_offsets.nepl, tests/stdlib/string_char.n.md"
---

# ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044: String float and char builder owner chains fail strict ResourceIR

## 概要

Focused string verification after alloc/string facade split reaches ResourceIR and reports resource.owner.use_after_move/reserved in from_f64_append_fraction_result, from_f64_build_fixed_result, string_char.n.md char slice checks, and ByteBuilder finish chains. The failures occur on Result-returning builder owners that should transfer exactly once through Ok arms.

## 対象

- `nepl-core/src/resource/owner_*.rs, stdlib/alloc/string/float.nepl, stdlib/alloc/string/slice.nepl, stdlib/alloc/string/char_offsets.nepl, tests/stdlib/string_char.n.md`

## 根拠

- `tests/stdlib/string_char.n.md::doctest#1` は `str_slice_chars_result` 成功後に source `str` を読むと `resource.owner.reserved` になっていた。
- `tests/stdlib/string_char.n.md::doctest#3` は multi-byte `byte_builder_push_char_utf8` 後の `byte_builder_finish` が `resource.owner.use_after_move` になっていた。
- stdlib-only の owner flow 書き換えでは `ByteBuilder` の owner-preserving source policy と衝突したため、根本原因を ResourceIR owner summary の nested `Result` variant propagation として切り分けた。

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

## 2026-05-07 Agent 2 final

原因:

- `OwnerReturnSummary` は direct `Result` arm の owner return / consume は扱えていたが、関数が別の `Result` から得た pending variant owner effect を戻り値 `Result` へ包み直す nested path では、variant ごとの owner return / consume summary を caller へ伝播していなかった。
- 同一 variant で source owner を payload として返している場合でも、pending consumption が残ると `match Result::Ok` 後に同じ owner を返却済みかつ消費済みとして扱い、`reserved` / `use_after_move` を発生させていた。
- `str_slice_chars_result` は start/end の 2 回変換で `Result` payload を作り、source `str` の raw-header/view summary に余計な owner state を通していた。

修正:

- `PendingVariantOwnerEffects::collect_result_owner_effect_summaries` を追加し、nested return 収集時に pending `Result` variant の owner consume / projection return を関数 summary へ伝播するようにした。
- `apply_match_arm` は同一 variant で返却された source owner を同じ arm の conditional consume として二重消費しないようにした。
- variant projection return source を unconditional consume 判定からも除外し、`Ok` で返る owner を関数全体の消費扱いにしないようにした。
- `str_char_byte_index_result` / `str_slice_chars_result` は `alloc/string/char_offsets.nepl` へ分離した offset helper を使い、`Result<CharUtf8Step, str>` payload を介さず byte offset を計算する。
- `ByteBuilder` は source policy が要求する owner-preserving `get_ref` / `byte_builder_with_len` 設計を維持した。stdlib を compiler bug 回避のための不自然な direct owned-ptr store へ曲げる変更は採用しなかった。
- `nodesrc/test_stdlib_string_slice_boundary.js` に `char_offsets` の責務境界と `StringUtf8LeadKind` の網羅 `match` を監視する policy を追加した。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_string_source_after -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_nested_byte_builder_result_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_byte_builder_owner_through_text_result_mapping -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_source_after_string_from_mem_copy -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md -i stdlib/alloc/string/float.nepl -i stdlib/alloc/string/builder.nepl -i stdlib/alloc/io.nepl --no-tree -o tmp/string-builder-owner-after-fix.json -j 1`: total=5, passed=5
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 193 passed / 10 failed。別 worktree の clean `origin/main@3ba24e72` でも同じ 10 件が失敗したため、今回差分由来ではない baseline issue として `ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F` を追加済み。
