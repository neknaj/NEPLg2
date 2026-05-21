---
id: ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC
title: "Collection slot drop lifecycle needs loaded-value drop proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/**"
---

# ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC: Collection slot drop lifecycle needs loaded-value drop proof

## 概要

Droppable collection slot lifecycle events are currently rejected unless state-only, because the checker has no generic evidence that a raw-loaded slot payload was actually dropped before DropInitialized or ReplaceDropOld advances slot state. Keeping this as a permanent rejection blocks non-Copy collection payload support; allowing it without proof would hide leaks or missing destructors.

## 対象

- `nepl-core/src/resource/**`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を stdlib 個別 allowlist ではなく Resource IR の generic proof boundary へ載せることを要求している。
- [ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2](./ISS-20260521T002920171Z-COLLECTION-SLOT-DROP-LIFECYCLE-CAN-E-DB699FC2.md) では、実 Drop proof がない droppable slot cleanup を安全側に拒否する guard だけを追加していた。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、collection slot lifecycle を compiler-core の typed proof boundary として扱い、module allowlist や raw `MemPtr` owner 復活を禁止している。

## 問題

Droppable collection slot lifecycle events are currently rejected unless state-only, because the checker has no generic evidence that a raw-loaded slot payload was actually dropped before DropInitialized or ReplaceDropOld advances slot state. Keeping this as a permanent rejection blocks non-Copy collection payload support; allowing it without proof would hide leaks or missing destructors.

## 影響

Non-Copy collection payload support cannot complete, and any future positive support for DropInitialized or ReplaceDropOld would be memory-unsafe unless it is tied to Resource IR value-flow and Drop proof instead of stdlib allowlists or state-only assertions.

## 修正方針

Track raw-loaded non-Copy values as Resource IR ownership origins, record a typed proof when that loaded value is consumed by ResourceOp::Drop, and require that proof for droppable CollectionSlotLifecycle DropInitialized and ReplaceDropOld. Summary replay must carry only callee-certified proof evidence.

## 修正内容

- `RawCellValueFlowFacts` に `DropLoadedCell` と raw-loaded value origin を追加し、raw load で materialize された non-Copy payload が `ResourceOp::Drop` / assignment overwrite auto-drop / scope auto-drop に到達した場合だけ drop proof を記録するようにした。
- raw-loaded value origin は `DeclareLocal`、`Read`、`Assign`、`Move`、aggregate construct、branch/match output transfer に追従し、call argument や raw store で別の ownership boundary へ移った場合は local drop proof として残さない。
- `CollectionSlotDropObligation` / `CollectionSlotDropProof` を追加し、`DropInitialized` と `ReplaceDropOld` は droppable payload の場合に `DropLoadedCell` proof を消費する。`ReplaceDropOld` は既存の new payload `StoreValue` proof も同時に要求する。
- `CollectionSlotLifecycleSummaryEventProof` を owner-transfer proof と slot-drop proof の構造体へ変更し、callee 内で証明済みの loaded-value drop だけを summary replay で caller に伝えるようにした。

## 検証

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core raw_cell_value_flow -- --test-threads=1`: passed
- `cargo test -p nepl-core initialized_collection_slot -- --test-threads=1`: passed
- `cargo test -p nepl-core collection_slot -- --test-threads=1`: passed
- `cargo test -p nepl-core resource_ir_collection_slot_drop --test resource_ir -- --test-threads=1`: passed
- `cargo test -p nepl-core resource_ir_collection_slot_replace_drop_old --test resource_ir -- --test-threads=1`: passed
- `cargo test -p nepl-core resource_ir_collection_slot_call_summary_accepts_callee_certified_drop_proof --test resource_ir -- --test-threads=1`: passed
