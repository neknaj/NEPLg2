---
id: ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE
title: "Collection slot drop traversal lowering lacks source coverage regression"
area: compiler
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260521T131741789Z-COLLECTION-SLOT-DROP-TRAVERSAL-LOWER-691CB7BE: Collection slot drop traversal lowering lacks source coverage regression

## 概要

ResourceOp::CollectionSlotDropTraversal has manual Resource IR tests, but the production lowering coverage regression only fixed ordinary lifecycle and storage relocate producers. A future source lowering refactor could stop emitting CollectionSlotDropTraversal while manual Resource IR tests remain green.

## 対象

- `nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- [ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B](./ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B.md) で、manual Resource IR 上の `CollectionSlotDropTraversal` proof と summary replay は固定された。
- [ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A](./ISS-20260521T062752580Z-COLLECTION-SLOT-LIFECYCLE-LOWERING-I-653BBF1A.md) は source lowering producer coverage を追加したが、fixture は ordinary lifecycle と storage relocate であり、drop traversal producer を個別に外した時の regression はなかった。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、stdlib 関数名 allowlist ではなく compiler-owned source evidence から Resource IR proof producer へ接続する方針を定めている。

## 問題

ResourceOp::CollectionSlotDropTraversal has manual Resource IR tests, but the production lowering coverage regression only fixed ordinary lifecycle and storage relocate producers. A future source lowering refactor could stop emitting CollectionSlotDropTraversal while manual Resource IR tests remain green.

## 影響

Non-Copy collection cleanup support could silently lose the source-level compiler-owned producer and drift back toward stdlib-specific cleanup assumptions.

## 修正方針

Add a source-level stdlib fixture that invokes collection_slot_drop_traversal, assert lowering emits ResourceOp::CollectionSlotDropTraversal, and assert ResourceLoweringCoverage reports CountMismatch when that producer is removed.

## 対応内容

- `resource_ir_lowering_coverage_guards_collection_slot_drop_traversal` を追加し、configured stdlib source の `#intrinsic "collection_slot_drop_traversal"` から `ResourceOp::CollectionSlotDropTraversal` が生成されることを検査した。
- 同 regression で、生成後の Resource IR から `CollectionSlotDropTraversal` だけを削除すると `ResourceCoverageKind::CollectionSlotLifecycle` の `CountMismatch` が出ることを固定した。
- これにより manual Resource IR tests が通っていても production source lowering の traversal producer が消えた場合に検出できる。

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_collection_slot_drop_traversal -- --test-threads=1
