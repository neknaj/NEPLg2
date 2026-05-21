---
id: ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36
title: "Collection slot BorrowRead lacks source-level lifecycle regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260521T064245727Z-COLLECTION-SLOT-BORROWREAD-LACKS-SOU-1999DF36: Collection slot BorrowRead lacks source-level lifecycle regression

## 概要

BorrowRead exists in CollectionSlotLifecycleEvent and lowering, but source-level compiler-owned stdlib tests do not prove that BorrowRead preserves initialized slot state and rejects moved or type-mismatched slots. Hand-level state tests are not enough to guard the production lowering path used by future non-Copy collection observers.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `CollectionSlotLifecycleEvent::BorrowRead` と lowering は存在し、手書き state transition test では initialized slot を状態変更なしで読めることを確認していた。
- ただし compiler-owned stdlib source lowering 経由では、`collection_slot_borrow_read` が typed lifecycle event として出ること、BorrowRead 後に initialized state が維持されること、MoveOut 後の BorrowRead が generic state refutation になることを固定していなかった。
- type argument / owner token anchor の mismatch は typecheck boundary で拒否されるため、Resource IR では source-level に到達する valid BorrowRead の lifecycle state を回帰対象にする。

## 問題

BorrowRead exists in CollectionSlotLifecycleEvent and lowering, but source-level compiler-owned stdlib tests do not prove that BorrowRead preserves initialized slot state and rejects moved or type-mismatched slots. Hand-level state tests are not enough to guard the production lowering path used by future non-Copy collection observers.

## 影響

A future collection observer implementation could regress BorrowRead lowering or checker behavior without an immediate source-level failure, weakening non-Copy collection payload safety before self-host collections are enabled.

## 修正方針

Add compiler-owned stdlib source regressions for BorrowRead: an initialized non-Copy slot remains initialized and can still be moved out with raw value-flow proof, and a moved slot rejects BorrowRead through the generic collection slot state refutation. Keep mismatched anchor/type rejection at the existing typecheck boundary.

## 対応

- `resource_ir_collection_slot_source_borrow_read_preserves_initialized_slot` を追加し、source-level `raw store -> InitializeEmpty -> BorrowRead -> raw load -> MoveOut` が diagnostic なしで通ることを固定した。
- `resource_ir_collection_slot_source_borrow_read_rejects_moved_slot` を追加し、source-level `MoveOut` 後の `BorrowRead` が `CollectionSlotLifecycleOp::BorrowRead` / `CollectionSlotState::Moved` の typed refutation になることを固定した。
- BorrowRead は owner-transfer / drop proof を消費せず、slot state の availability/type precondition だけを汎用 `CollectionSlotStateTable` で検査することを、production lowering path から確認した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_borrow_read -- --test-threads=1`: passed
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`: passed
