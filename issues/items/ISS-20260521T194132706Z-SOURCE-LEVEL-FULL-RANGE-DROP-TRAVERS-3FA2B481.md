---
id: ISS-20260521T194132706Z-SOURCE-LEVEL-FULL-RANGE-DROP-TRAVERS-3FA2B481
title: "Source-level full-range drop traversal proof needs end-to-end regression"
area: compiler-core
status: fixed
resolved: true
priority: P0
type: test
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/tests, nepl-core/src/resource/**"
---

# ISS-20260521T194132706Z-SOURCE-LEVEL-FULL-RANGE-DROP-TRAVERS-3FA2B481: Source-level full-range drop traversal proof needs end-to-end regression

## 概要

ForallInitializedRange drop traversal proof is covered by synthetic ResourceOp unit tests, but the compiler-owned source path does not yet have an end-to-end regression showing that a while loop over initialized_len produces the generic full-range certificate, nor negative coverage for incomplete loops.

## 対象

- `nepl-core/tests, nepl-core/src/resource/**`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、非 Copy collection cleanup を stdlib allowlist ではなく Resource IR の typed certificate / Resource IR evidence として証明する方針を定めている。
- `ForallInitializedRange` summary certificate は synthetic ResourceOp test だけでは source lowering、alias propagation、caller-side storage release までの接続を保証できなかった。

## 問題

ForallInitializedRange drop traversal proof is covered by synthetic ResourceOp unit tests, but the compiler-owned source path does not yet have an end-to-end regression showing that a while loop over initialized_len produces the generic full-range certificate, nor negative coverage for incomplete loops.

## 影響

Non-Copy collection cleanup cannot safely leave the Copy-only policy until source-level stdlib code proves full initialized range traversal through Resource IR. Without this regression, future changes could silently fall back to finite markers or accept incomplete cleanup.

## 修正方針

Add source-level compiler-owned regression tests that lower raw load/drop loop traversal from NEPL source through Resource IR and require ForallInitializedRange-compatible cleanup; reject incomplete traversal patterns without stdlib allowlists.

## 検証

Run focused nepl-core source-level collection slot traversal tests and Resource IR collection slot summary tests.

## 対応

- Source-level regression を追加し、NEPL source から raw load / actual drop / loop induction / summary replay / caller-side storage dealloc までを通して検査するようにした。
- `i = 0; while i < initialized_count; i = i + 1` という全域 traversal だけを full range proof として受け入れ、非ゼロ開始と step two は拒否する負例で固定した。
- loop body candidate extraction は `ResourceOffset::Symbolic` / `ScaledSymbolic` と scalar alias fact を伝播しながら、source lowering で生成される byte-offset expression を storage + index + stride に戻す。
- caller replay では summary-certified drop traversal が raw cell alias と collection slot storage alias の両方を消費し、storage release も同じ alias set で未解放 slot を検査するようにした。
- owner-cell canonicalization は storage offset identity を raw mem-ptr field-only local より優先し、source lowering 由来の storage identity が certificate boundary で失われないようにした。
- 実装は stdlib module allowlist や helper 名の特別扱いではなく、Resource IR の source-derived loop / scalar / raw alias / collection slot evidence を使う汎用 proof path とした。

## 回帰テスト

- `source_loop_drop_traversal_summary_cleans_caller_initialized_range`
- `source_loop_drop_traversal_rejects_non_zero_start_as_full_range_proof`
- `source_loop_drop_traversal_rejects_step_two_as_full_range_proof`
- `collection_slot_summary_loop_certificate_survives_post_loop_anchor_read`
- `raw_value_flow_alias_matching_normalizes_scaled_symbolic_offsets`
- `owner_cell_canonicalization_prefers_storage_offset_identity`
- `raw_move_marks_alias_cells_moved`

## 検証結果

- `cargo test -p nepl-core --test collection_slot_full_range -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --test-threads=1`
- `cargo test -p nepl-core --lib raw_value_flow_alias_matching_normalizes_scaled_symbolic_offsets -- --test-threads=1`
- `cargo test -p nepl-core --lib owner_cell_canonicalization_prefers_storage_offset_identity -- --test-threads=1`
- `cargo test -p nepl-core --lib raw_move_marks_alias_cells_moved -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
