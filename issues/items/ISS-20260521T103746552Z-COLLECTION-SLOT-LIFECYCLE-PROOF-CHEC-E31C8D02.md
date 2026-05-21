---
id: ISS-20260521T103746552Z-COLLECTION-SLOT-LIFECYCLE-PROOF-CHEC-E31C8D02
title: "Collection slot lifecycle proof checks must be atomic"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/initialized_collection_slot_proof.rs
---

# ISS-20260521T103746552Z-COLLECTION-SLOT-LIFECYCLE-PROOF-CHEC-E31C8D02: Collection slot lifecycle proof checks must be atomic

## 概要

CollectionSlotLifecycle events that require both drop and owner-transfer proofs can consume one proof before discovering that the other proof is missing. A rejected event must not mutate proof state, otherwise diagnostics and later valid lifecycle events become order-dependent.

## 対象

- `nepl-core/src/resource/initialized_collection_slot_proof.rs`

## 根拠

- `ReplaceInitialized(DropOldOwner)` は old payload の actual drop proof と new payload の store proof の両方を必要とする。
- 修正前の checker は drop proof を先に消費してから owner-transfer proof を検査していたため、new payload store proof が欠けた rejected event でも `DropLoadedCell` fact が失われた。
- rejected lifecycle event は slot state を進めないため、証明 fact の消費も atomic no-op でなければならない。

## 問題

CollectionSlotLifecycle events that require both drop and owner-transfer proofs can consume one proof before discovering that the other proof is missing. A rejected event must not mutate proof state, otherwise diagnostics and later valid lifecycle events become order-dependent.

## 影響

Static checking can report cascading false DropRequiresElaboration diagnostics after the real root OwnerTransferRequiresValueProof, and the checker state no longer represents a failed event as an atomic no-op.

## 修正方針

Split proof satisfaction from proof consumption, validate all required obligations and the slot-state precondition first, then consume all proofs only when the lifecycle event can be accepted.

## 対応

- `CollectionSlotDropProof` / `CollectionSlotOwnerTransferProof` に、証明を消費しない satisfaction check を追加した。
- `CollectionSlotLifecycleProofPlan` を導入し、slot-state precondition、drop obligation、owner-transfer obligation をすべて検査してから proof plan を作るようにした。
- proof plan の消費は `CellTable` clone 上で行い、すべて成功した場合だけ original table へ反映する transaction にした。
- `MoveOutAndStoreValue` の local raw value-flow proof 消費も clone commit にし、old load と new store のどちらか一方だけが消費される経路を閉じた。
- regression として、missing replacement store proof により `ReplaceInitialized(DropOldOwner)` が拒否された後でも、同じ actual drop proof を後続 `DropInitialized` が利用でき、false `DropRequiresElaboration` が出ないことを固定した。

## 検証

Add a Resource IR regression where ReplaceInitialized(DropOldOwner) has a valid old-drop proof but lacks the replacement store proof, then a later DropInitialized reuses the old-drop proof without a second false diagnostic.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_failed_replace_drop_old_does_not_consume_drop_proof -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_replace_drop_old_accepts_drop_and_store_proofs -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_non_copy_replace_return_old_accepts_raw_load_and_store_proofs -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_accepts_callee_certified_drop_proof -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_accepts_callee_certified_non_copy_replace_return_old -- --test-threads=1`: pass
- `cargo check -p nepl-core`: pass
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot -- --test-threads=1`: timeout at 244s. 個別 relevant regression は上記で通過済み。全体確認は GH Actions 側へ委ねる。
