---
id: ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9
title: "Source-level droppable collection slot lifecycle lacks loaded-value drop regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9: Source-level droppable collection slot lifecycle lacks loaded-value drop regression

## 概要

Manual Resource IR tests cover DropInitialized and ReplaceDropOld loaded-value drop proof, but compiler-owned stdlib source fixtures do not yet prove that raw load, actual drop, and collection slot drop lifecycle lower to the same generic proof boundary.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- [ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC](./ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC.md) で、手書き Resource IR 上の `DropInitialized` / `ReplaceDropOld` は `DropLoadedCell` proof を要求・消費するようになった。
- [ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF](./ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF.md) で、source-level compiler-owned stdlib lowering の raw store/load fact は collection slot lifecycle target と alias-aware に照合されるようになった。
- ただし `DropInitialized` / `ReplaceDropOld` については、source lowering 経由で raw load、actual drop、lifecycle event が同じ generic proof boundary に到達する regression が未整備だった。

## 問題

Manual Resource IR tests cover DropInitialized and ReplaceDropOld loaded-value drop proof, but compiler-owned stdlib source fixtures do not yet prove that raw load, actual drop, and collection slot drop lifecycle lower to the same generic proof boundary.

## 影響

Non-Copy collection cleanup could remain correct only in manual Resource IR fixtures while production source lowering fails to connect loaded payload drops to collection slot lifecycle events.

## 修正方針

Add source-level compiler-owned stdlib Resource IR regressions for droppable non-Copy collection slot load/drop/DropInitialized and ReplaceDropOld using generic raw value-flow proof, without stdlib function allowlists.

## 対応

- `resource_ir_collection_slot_source_drop_initialized_accepts_actual_loaded_value_drop` を追加し、compiler-owned stdlib source path で `raw store -> InitializeEmpty -> raw load -> assignment overwrite drop -> DropInitialized` が diagnostics なしで通ることを固定した。
- `resource_ir_collection_slot_source_drop_initialized_rejects_raw_load_without_drop` を追加し、raw load だけでは droppable payload の `DropInitialized` を証明できず、actual drop proof が必須であることを固定した。
- `resource_ir_collection_slot_source_replace_drop_old_accepts_drop_and_store_proofs` を追加し、`ReplaceDropOld` が old payload の actual drop proof と new payload の raw store proof の両方を source-level lowering 経由で消費することを固定した。
- `resource_ir_collection_slot_source_replace_drop_old_rejects_missing_drop_or_store_proof` を追加し、old drop proof または new store proof のどちらかが欠けても lifecycle state を進めないことを固定した。
- いずれも stdlib module 名・関数名の allowlist ではなく、`ResourceOp::CollectionSlotLifecycle`、`RawCellValueFlowFacts`、`CollectionSlotDropProof` の generic proof boundary を通る。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_replace_drop_old -- --test-threads=1`: passed
