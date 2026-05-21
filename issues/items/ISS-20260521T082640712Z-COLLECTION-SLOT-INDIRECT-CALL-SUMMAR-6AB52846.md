---
id: ISS-20260521T082640712Z-COLLECTION-SLOT-INDIRECT-CALL-SUMMAR-6AB52846
title: "Collection slot indirect call summaries lose path-correlated state"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/{initialized.rs,initialized_control.rs,initialized_path_state.rs,collection_slot_summary_apply.rs,function_alias.rs}"
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

Introduce a path-correlated `ResourceCheckState` design for the state components that participate in indirect call summary application. Indirect call summary replay and summary translation must run per feasible path state and merge the resulting states, rather than applying every merged callee to one merged raw/slot state.

設計単位:

- `ResourceCheckState`: `CellTable`、`CollectionSlotStateTable`、`RawCellAddressAliases`、`FunctionAliasTable`、`PendingRawReallocs`、`PendingVariantRawCellInitializations` を束ねる。
- `ResourcePathAlternatives`: `None` / `Feasible(Vec<ResourceCheckState>)` を enum として保持し、path-sensitive consumer が exhaustive match で分岐できるようにする。
- branch / match / loop の join では、必要な consumer が path alternatives を使えるようにし、indirect call summary replay は path ごとに callee alias と raw/slot state を対応させてから結果を merge する。
- `CollectionSlotLifecycleSummaryOp::Merge` は callee summary 内の path を表す用途として維持するが、caller 側の branch 後 indirect call には実行中 checker state の path correlation も必要である。

実装単位:

1. path-correlated state と alternatives enum の型を追加し、linear op を feasible path alternatives ごとに進めてから merge する。
2. branch / match join 後の alternatives に branch value / match value の transfer も path ごとに反映し、nested branch/match の alternatives が別 arm へ漏れないようにする。
3. branch / match join 後の indirect call regression を追加し、実行不能な callee/state cross product で false positive が出ないことを固定する。
4. `FunctionAliasTable::merge_paths` は汎用 may-join として残し、path-sensitive consumer は `ResourceCheckState` alternatives 経由で callee alias と raw/slot state を対応させる。

## 修正内容

- `nepl-core/src/resource/initialized_path_state.rs` を追加し、Resource IR checker の path-sensitive state を `ResourceCheckState` として束ね、alternatives の有無を `ResourcePathAlternatives` enum で表した。
- `ResourceCheckEngine::check_ops` は alternatives が存在する場合、merged state を診断元にせず、各 feasible path に同じ `ResourceOp` を適用してから merge するようにした。これにより callee alias と collection slot state の実行不能な cross product を作らない。
- branch / match は arm 内の nested alternatives を個別に捕捉し、branch value / match value の consume、raw alias rekey、collection slot transfer、function alias、pending realloc、variant initialization を path ごとに反映してから merge するようにした。
- loop 内で生じた path alternatives は loop の既存 conservative merge に閉じ込め、body 側 alternatives が loop 後の別 op へ漏れないようにした。
- path-specific checker で生じた diagnostics / auto drop points / deferred merge count は親 checker へ吸収し、merged-state skip によって診断を失わないようにした。
- stdlib module 名、関数名、型名の allowlist は追加していない。すべて Resource IR state と summary proof を基に処理する。

## 検証

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_indirect_return_summary_preserves_path_correlation -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_indirect_call_summary_applies_function_alias -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_branch_path_alternatives_do_not_keep_invalid_output_initialized -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_indirect -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_ -- --test-threads=1`: timed out locally at 244s, so local verification is narrowed to the directly affected summary groups and full broad verification is left to GitHub Actions.
