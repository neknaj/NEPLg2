---
id: ISS-20260520T165159064Z-RAW-CELL-LIFECYCLE-SOURCE-POLICY-ONL-848A217D
title: "Raw cell lifecycle source policy only checks surface strings"
area: tools
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-20
updated: 2026-05-21
target: "nodesrc/test_resource_raw_cell_lifecycle_policy.js, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260520T165159064Z-RAW-CELL-LIFECYCLE-SOURCE-POLICY-ONL-848A217D: Raw cell lifecycle source policy only checks surface strings

## 概要

The raw lifecycle source policy checks event names, a match string, and a small forbidden-call list. It passes even when semantic Resource IR regressions such as realloc range transfer failures remain.

## 対象

- `nodesrc/test_resource_raw_cell_lifecycle_policy.js, nepl-core/tests/resource_ir.rs`

## 根拠

- `nodesrc/test_resource_raw_cell_lifecycle_policy.js` は lifecycle event 名と一部の文字列だけを見ており、`RawCellLifecycleEvent` の後条件が Resource IR 回帰で守られているかを十分に監視していなかった。
- `copy_initialized_copy_raw_cells_covered_by_count` / `copy_initialized_raw_byte_ranges_for_bulk_copy` / `extend_initialized_raw_byte_ranges` などの直接呼び出しが増えても、責務分割と bypass 検出が追従していなかった。
- `node nodesrc/test_resource_checker_responsibility.js` を通すと、未監視ファイルや責務上限の不整合が複数検出された。

## 問題

The raw lifecycle source policy checks event names, a match string, and a small forbidden-call list. It passes even when semantic Resource IR regressions such as realloc range transfer failures remain.

## 影響

A policy test can give false confidence that lifecycle proof was centralized while pre/postconditions are still incomplete. This is especially risky for memory-safety work because enum existence is not enough to prove transition correctness.

## 修正方針

Keep the nodesrc test as an architecture smoke test, but add semantic regression requirements around each lifecycle postcondition and broaden mutation-bypass detection so source policy cannot be the only guard.

## 対応

- raw lifecycle source policy に、現行の `BulkCopyInitializedRawState` / `CopyRawElementType` / count proof helper / semantic Resource IR regression 名を要求する検査を追加した。
- source policy が意味論の正とならないよう、move / store reinitialize / raw fill / bulk copy / realloc / dealloc / stale range invalidation の回帰テスト名を要求し、Rust 側の実テストが semantic authority である構成にした。
- `CellTable` の raw copy helper、raw range copy helper、cell state unit tests、i32 call facts scale helper、raw memory bulk handler を責務単位の別モジュールへ分割した。
- `nodesrc/test_resource_checker_responsibility.js` に新規 resource module の監視を追加し、既存の専用責務ファイルは現実的な上限へ調整した。
- 作業中に `byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local` の既存失敗を検出したため、`ISS-20260520T180237730Z-INITIALIZED-RAW-RANGE-COUNT-ALIASES--A1FBF011` として別 issue 化した。

## 検証

- `node nodesrc/test_resource_raw_cell_lifecycle_policy.js`
- `node nodesrc/test_resource_checker_responsibility.js`
- `cargo test -p nepl-core copy_store_preserves_unknown_offset_initialized_copy_fact -- --test-threads=1`
- `cargo test -p nepl-core raw_move_clears_overlapping_initialized_raw_byte_range -- --test-threads=1`
- `cargo test -p nepl-core raw_move_clears_overlapping_initialized_raw_cell_entry -- --test-threads=1`
- `cargo test -p nepl-core i32_call_facts -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_bulk -- --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized -- --test-threads=1`
