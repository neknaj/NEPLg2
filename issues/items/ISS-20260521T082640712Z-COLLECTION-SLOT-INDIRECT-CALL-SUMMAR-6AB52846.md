---
id: ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846
title: "Collection slot indirect call summaries lose path-correlated state"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/{initialized_control.rs,collection_slot_summary_apply.rs,collection_slot_summary_translate.rs,function_alias.rs}"
---

# ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846: Collection slot indirect call summaries lose path-correlated state

## 概要

Branch and match joins merge FunctionAliasTable, raw aliases, cells, collection slot state, pending reallocs, and variant initialization independently. After the join, an indirect call can combine a callee alias from one path with slot/raw state from another path.

## 対象

- `nepl-core/src/resource/{initialized_control.rs,collection_slot_summary_apply.rs,collection_slot_summary_translate.rs,function_alias.rs}`

## 根拠

- `FunctionAliasTable::merge_paths` は place ごとの callee 候補を union する。
- `RawCellAddressAliases::merge_paths`、`CellTable::merge_paths_with_raw_aliases`、`CollectionSlotStateTable::merge_paths`、`PendingRawReallocs::merge_paths`、`PendingVariantRawCellInitializations::merge_paths` はそれぞれ独立に join される。
- `ResourceCheckEngine::apply_indirect_call_collection_slot_lifecycle_summary` は join 後の `FunctionAliasTable` から得た全 callee に対して、同じ join 済み raw alias / collection slot / cell state を使って summary を replay する。
- そのため「path A でだけ成り立つ callee」と「path B でだけ成り立つ live slot state」が cross product として組み合わされ、実行不能な transfer が作られる。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 は、個別 allowlist ではなく Resource IR state による汎用 proof boundary を要求している。path correlation はこの proof state の一部として扱う必要がある。

## 問題

Branch and match joins merge FunctionAliasTable, raw aliases, cells, collection slot state, pending reallocs, and variant initialization independently. After the join, an indirect call can combine a callee alias from one path with slot/raw state from another path.

## 影響

The first concrete failure mode is false positive collection-slot diagnostics for execution-impossible callee/state combinations, and the same lossy state product makes future static-check changes fragile around memory-safety proof summaries.

## 修正方針

Introduce a path-correlated ResourceCheckState/ResourcePathState design for the state components that participate in indirect call summary application. Indirect call summary replay and summary translation must run per feasible path state and merge the resulting states, rather than applying every merged callee to one merged raw/slot state.

設計単位:

- `ResourceCheckState`: `CellTable`、`CollectionSlotStateTable`、`RawCellAddressAliases`、`FunctionAliasTable`、`PendingRawReallocs`、`PendingVariantRawCellInitializations` を束ねる。
- `ResourcePathState`: `Single(ResourceCheckState)` / `Merge(Vec<ResourcePathState>)` のように feasible path alternatives を保持する。
- branch / match / loop の join では、必要な consumer が path alternatives を使えるようにし、indirect call summary replay は path ごとに callee alias と raw/slot state を対応させてから結果を merge する。
- `CollectionSlotLifecycleSummaryOp::Merge` は callee summary 内の path を表す用途として維持するが、caller 側の branch 後 indirect call には実行中 checker state の path correlation も必要である。

最初の実装単位:

1. path-correlated state の型を追加し、collection slot indirect call summary replay の入口を path ごとに処理できる形へ分離する。
2. branch / match join 後の indirect call regression を追加し、実行不能な callee/state cross product で false positive が出ないことを固定する。
3. `FunctionAliasTable::merge_paths` は汎用 may-join として残してよいが、path-sensitive consumer では直接使わない。

## 検証

Add a regression where one branch pairs a live slot with a fresh-storage callee and the other pairs an empty slot with an identity callee; storage deallocation of the indirect result must not report a live-slot diagnostic.
