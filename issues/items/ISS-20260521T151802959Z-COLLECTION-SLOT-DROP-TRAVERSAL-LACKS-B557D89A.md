---
id: ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A
title: "Collection slot drop traversal lacks typed initialized count operand"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_collection_slot.rs, nepl-core/src/resource/collection_slot_drop_traversal.rs"
---

# ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A: Collection slot drop traversal lacks typed initialized count operand

## 概要

ResourceOp::CollectionSlotDropTraversal and the collection_slot_drop_traversal intrinsic only carry storage and element type, so the compiler has no typed operand representing the initialized range length that a dynamic traversal proof must cover.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_collection_slot.rs, nepl-core/src/resource/collection_slot_drop_traversal.rs`

## 根拠

- `ResourceOp::CollectionSlotDropTraversal` が storage と expected type だけを持つと、`initialized_len` / element count と traversal proof を結ぶ typed source fact が存在しない。
- `collection_slot_drop_traversal` intrinsic の arity も storage だけだったため、source lowering から initialized count を Resource IR / summary へ通せなかった。
- 前段の [ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183](./ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183.md) で symbolic slot を拒否したため、positive な dynamic cleanup へ進むには count operand が必須になっていた。

## 問題

ResourceOp::CollectionSlotDropTraversal and the collection_slot_drop_traversal intrinsic only carry storage and element type, so the compiler has no typed operand representing the initialized range length that a dynamic traversal proof must cover.

## 影響

Dynamic OwnedBuffer or Vec cleanup cannot be proven from source range facts without adding ad hoc stdlib-specific knowledge or reusing a single symbolic slot as if it represented the full initialized range.

## 修正方針

Extend the traversal boundary to carry an initialized-count Place from source lowering through Resource IR and summaries. Keep explicit finite-slot cleanup behavior, but make the count operand available for later generic range proof checks.

## 実装

- `CollectionSlotDropTraversal` に `initialized_count: Place` を追加し、source lowering、initialized checker dispatch、dump、coverage、borrow usage、temporary scope、effect summary seed walk を更新した。
- `collection_slot_drop_traversal<T>` intrinsic は `(storage, initialized_count)` を受け取り、typecheck 境界で count が `i32` であることを検査する。
- `CollectionSlotLifecycleSummaryOp::DropTraversal` にも `initialized_count` を通し、summary build / translate / replay が count を捨てないようにした。
- known-offset slot について、`offset / element_stride < initialized_count` を `RawCellAddressAliases` の i32 relation fact で証明できる場合だけ traversal を進める。zero-sized payload は byte stride が存在しないため、明示的 known slot に対して count が正であることを要求する。
- symbolic / unknown offset は引き続き full range proof が未実装なので `RangeProofRequired` のまま拒否する。

## 検証

Add/update Resource IR lowering and source-level tests so collection_slot_drop_traversal requires and preserves the initialized count operand.

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test effects collection_slot_drop_traversal -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_collection_slot_drop_traversal -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_drop_traversal -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_traversal_storage_release -- --test-threads=1`: passed
- `cargo test -p nepl-core --lib collection_slot -- --test-threads=1`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
