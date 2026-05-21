---
id: ISS-20260521T143810964Z-COLLECTION-SLOT-DROP-TRAVERSAL-SUMMA-131D4BA0
title: "Collection slot drop traversal summary certifies marker-only cleanup"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_build_ops.rs, nepl-core/src/resource/collection_slot_summary_replay.rs, nepl-core/src/resource/collection_slot_drop_traversal.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260521T143810964Z-COLLECTION-SLOT-DROP-TRAVERSAL-SUMMA-131D4BA0: Collection slot drop traversal summary certifies marker-only cleanup

## 概要

CollectionSlotDropTraversal summary build can emit CertifiedLoadedValueDrops when the callee summary state has no tracked initialized slots, so a marker-only helper body can be replayed by the caller as if it had dropped every caller slot.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_ops.rs, nepl-core/src/resource/collection_slot_summary_replay.rs, nepl-core/src/resource/collection_slot_drop_traversal.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `CollectionSlotSummaryBuildState::new` は function parameter の raw/cell alias だけを seed し、callee の collection slot table は空から始まる。
- 旧 `collection_slot_drop_traversal_available` は空の slot set に対しても `Ok(())` を返したため、marker-only helper body から `CertifiedLoadedValueDrops` summary を生成できた。
- 旧 summary replay は caller 側の storage 配下 slot を列挙し、callee が実際に証明した slot 範囲を持たないまま `SummaryCertified` proof を各 caller slot に適用していた。

## 問題

CollectionSlotDropTraversal summary build can emit CertifiedLoadedValueDrops when the callee summary state has no tracked initialized slots, so a marker-only helper body can be replayed by the caller as if it had dropped every caller slot.

## 影響

A helper call can make caller-owned non-Copy collection slots look dropped without source-derived raw load and Drop evidence. That hides missing cleanup proof and can incorrectly permit later storage dealloc, violating the Resource IR static-check safety boundary.

## 修正方針

Make drop traversal summaries carry source-derived certified slot targets and replay only those slots. Do not emit a collection-wide traversal summary from a marker-only or vacuous callee state; caller storage dealloc must still reject remaining live slots.

## 対応

- `CollectionSlotLifecycleSummaryOp::DropTraversal` に `certified_slots` を追加し、callee summary build が実際に local loaded-value drop proof で証明できた slot target だけを summary payload に保持するようにした。
- marker-only / 空の traversal は summary op を生成しない。caller 側の live slot は live のまま残り、後続の storage dealloc が typed refutation になる。
- summary replay は storage 全体を再列挙しない。summary payload の `certified_slots` だけを instantiation し、storage prefix に含まれることを検査してから `SummaryCertified(DropLoadedValue)` を適用する。
- 旧 `CollectionSlotDropTraversalProof::SummaryCertified` と `collection_slot_drop_traversal_available` を削除し、collection-wide marker に戻る入口を閉じた。
- summary translate の DropTraversal 分岐を専用 module に分割し、responsibility policy の上限緩和を避けた。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_drop_traversal_summary -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir collection_slot_drop_traversal -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js check --dir issues`: passed
