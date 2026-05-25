---
id: ISS-20260525T132633255Z-VEC-FILTER-RETURN-RANGE-CLEANUP-LOSE-6D89D246
title: "Vec filter return range cleanup loses equal initialized count projections"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-25
updated: 2026-05-25
target: "nepl-core/src/resource/i32_scalar_return_facts.rs; nepl-core/src/resource/collection_slot_summary_return_path_value.rs; nepl-core/src/resource/collection_slot_summary_apply_return_path.rs"
---

# ISS-20260525T132633255Z-VEC-FILTER-RETURN-RANGE-CLEANUP-LOSE-6D89D246: Vec filter return range cleanup loses equal initialized count projections

## 概要

Vec filter returning an OwnedBuffer can publish the same output initialized range with len and initialized_len count projections, but the caller did not receive the equality between those return-value projections.

## 対象

- `nepl-core/src/resource/i32_scalar_return_facts.rs; nepl-core/src/resource/collection_slot_summary_return_path_value.rs; nepl-core/src/resource/collection_slot_summary_apply_return_path.rs`

## 根拠

- `resource_ir_vec_filter_drop_payload_uses_transform_range_certificate` は、修正前に `LiveSlotDuringStorageDealloc` で失敗していた。
- 残っていた live range は同じ output storage に対する range で、片方の count が returned `OwnedBuffer.len`、もう片方の count が returned `OwnedBuffer.initialized_len` だった。
- filter の成功経路ではどちらも同じ write index を表すが、return path scalar fact が戻り値内の等価射影を伝播していなかったため、range cleanup が片方だけを消していた。

## 問題

Vec filter returning an OwnedBuffer can publish the same output initialized range with len and initialized_len count projections, but the caller did not receive the equality between those return-value projections.

## 影響

Drop traversal can clear the initialized_len range while leaving the equivalent len range live, so storage deallocation reports LiveSlotDuringStorageDealloc even after the transform range source/output lifecycle is otherwise proven.

## 修正方針

Preserve i32 relations between equal return-value leaf projections and collect scalar facts from completed construct outputs so len and initialized_len equality reaches caller-side range cleanup.

## 検証

Run the focused i32 scalar relation unit test and resource_ir_vec_filter_drop_payload_uses_transform_range_certificate.

## 2026-05-25 修正結果

`I32ScalarReturnFacts` に戻り値内 i32 relation を追加し、等価な return-value leaf projection を caller の `RawCellAddressAliases` へ復元するようにした。

また、construct の field ごとの return path だけでは `OwnedBuffer.len` と `OwnedBuffer.initialized_len` の関係を見られないため、construct 完了後の出力全体から scalar fact を収集する経路を追加した。return path 適用時は、return range を復元する前に i32 scalar fact を適用し、range count の等価性を `mark_initialized_range_with_aliases` から参照できるようにした。

独立レビューで、range cleanup が `initialized_count` を見ずに storage 配下の concrete slot を消せる点と、construct return path merge が precondition の異なる path を `ops` だけで合流できる点を unsound として確認した。対応として、concrete slot の正規化は count 内と証明できる slot に限定し、construct return path は `ops` / `preconditions` / `return_variant` が一致する場合だけ合流するようにした。

検証:

- `cargo fmt -p nepl-core --check`: passed.
- `cargo test -p nepl-core i32_return_facts_preserve_equal_return_leaf_relations --lib -- --nocapture`: passed.
- `cargo test -p nepl-core clearing_initialized_range_uses_i32_relation_for_count_equivalence --lib -- --nocapture`: passed.
- `cargo test -p nepl-core i32_condition_uses_exact_value_derived_from_offset_chain --lib -- --nocapture`: passed.
- `cargo test -p nepl-core --lib -- --nocapture`: 370/370 passed.
- `cargo test -p nepl-core --test resource_ir resource_ir_vec_filter_drop_payload_uses_transform_range_certificate -- --exact --nocapture`: passed, 434.45s.
- `cargo test -p nepl-core --test collection_slot_full_range -- --nocapture`: 8/8 passed, 54.00s.
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_traversal_storage_release -- --exact --nocapture`: passed, 54.23s.
- `node nodesrc/issues.js check --dir issues`: passed.
- `node nodesrc/neplg21_syntax_migrate.js --check`: would update 0 file(s).
- `git diff --check`: passed. CRLF checkout warning のみ。
- `trunk build`: passed.
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests_checkpoint-resource-return-range.json`: 13/13 passed, JSON の `failedCount=0` を確認。

残検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle -- --exact --nocapture` は 300s timeout。今回の exact filter は通過しており、stdlib Vec integration の長時間化は `ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5` の性能対象として継続する。
