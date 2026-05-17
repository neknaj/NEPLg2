---
id: ISS-20260517T182739351Z-RESOURCE-IR-REJECTS-INITIALIZED-NEST-1388C7B5
title: "Resource IR rejects initialized nested Copy field loaded from raw aggregate"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/resource/cell_state.rs; nepl-core/tests/resource_ir.rs; doc/neplg2/static_check_complexity_reduction_plan.md
---

# ISS-20260517T182739351Z-RESOURCE-IR-REJECTS-INITIALIZED-NEST-1388C7B5: Resource IR rejects initialized nested Copy field loaded from raw aggregate

## 概要

A Copy aggregate copied out of initialized storage can be marked initialized as a whole while a later nested field access lowers through a raw cell path such as local + storage offset + deref + field. Cell availability only flows from whole-value initialization through ordinary field projections, so the nested raw-cell field is reported as resource.cell.uninit even though the source value is initialized and Copy.

## 対象

- `nepl-core/src/resource/cell_state.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- selfhost module loader doctest の `let span_file_id <i32> item.span.file_id` が `resource.cell.uninit` で失敗した。
- focused regression でも `load<Item>` 後の `item.span.file_id` が `Local("item") + StorageOffset(4) + Deref + Field(0)` として現れ、whole-value initialized fact から field-storage view へ証明が流れないことを再現した。

## 問題

A Copy aggregate copied out of initialized storage can be marked initialized as a whole while a later nested field access lowers through a raw cell path such as local + storage offset + deref + field. Cell availability only flows from whole-value initialization through ordinary field projections, so the nested raw-cell field is reported as resource.cell.uninit even though the source value is initialized and Copy.

## 影響

Self-host module loader doctests that inspect SelfhostModuleItem.span.file_id fail under Resource IR. More generally, safe nested field reads from Copy aggregates can be rejected or may force source rewrites that hide a compiler proof gap.

## 修正方針

Teach the initialized-cell proof to relate initialized whole Copy aggregate values and raw-cell projections that read fields from their storage address without weakening non-Copy move/drop checks. Add a focused regression that reads an i32 leaf through a nested Copy struct field after loading the outer struct from raw memory.

## 検証

cargo test -p nepl-core resource_ir_cell_check_preserves_nested_copy_field_after_raw_aggregate_load -- --nocapture; cargo check -p nepl-core --tests

## 対応内容

- `CellTable::availability_state_with_types` が `TypeCtx` を保持したまま initialized fact の flow 判定に進めるようにした。
- `initialized_storage_view_flows_to` を追加し、known storage offset が aggregate layout 上の field と一致し、残りの projection の最終 query type が Copy の場合だけ initialized fact を流すようにした。
- unknown / symbolic offset、追加 deref、non-Copy query は許可しないため、raw memory 全般を緩めず、compiler が生成した typed field-storage view の Copy leaf 読み出しだけを証明する。
- `resource_ir_cell_check_preserves_nested_copy_field_after_raw_aggregate_load` を追加し、raw-loaded Copy aggregate からの `item.span.file_id` 読み出しを regression として固定した。
- `doc/neplg2/static_check_complexity_reduction_plan.md` に Stage 4 initialized cell proof の進捗として追記した。

## 検証結果

- `cargo test -p nepl-core resource_ir_cell_check_preserves_nested_copy_field_after_raw_aggregate_load -- --nocapture`: passed
