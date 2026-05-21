---
id: ISS-20260521T163555637Z-COLLECTION-SLOT-SUMMARY-CANNOT-INSTA-02D58E62
title: "Collection slot summary cannot instantiate symbolic offset operands across calls"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_target.rs
---

# ISS-20260521T163555637Z-COLLECTION-SLOT-SUMMARY-CANNOT-INSTA-02D58E62: Collection slot summary cannot instantiate symbolic offset operands across calls

## 概要

CollectionSlotLifecycleSummaryPlace stored only a parameter index plus a raw PlaceProjection suffix. When the suffix contained PlaceProjection::StorageOffset(ResourceOffset::Symbolic or ScaledSymbolic), the embedded Place remained a callee-local operand. instantiate_summary_target attached the suffix to the caller argument but did not recursively instantiate those embedded Places.

## 対象

- `nepl-core/src/resource/collection_slot_summary_target.rs`
- `nepl-core/src/resource/collection_slot_summary_projection.rs`
- `nepl-core/src/resource/collection_slot_summary_return_*.rs`

## 根拠

- `CollectionSlotLifecycleSummaryProjection` / `CollectionSlotLifecycleSummaryOffset` を追加し、summary suffix と return suffix から raw `PlaceProjection` を排除した。
- `ResourceOffset::Symbolic` / `ScaledSymbolic` の operand は `CollectionSlotLifecycleSummaryPlace` として parameter-relative に要約される。parameter に相対化できない operand は summary 化せず、callee-local place を caller replay へ持ち越さない。
- return transfer / return slot の suffix も同じ typed suffix を使い、wrapper summary composition では callee suffix を caller argument へ instantiate してから wrapper parameter-relative に再要約する。
- 回帰テスト `collection_slot_summary_target_*` で、scaled symbolic operand の caller argument 置換、非 parameter operand の拒否、wrapper return suffix translation を固定した。

## 問題

CollectionSlotLifecycleSummaryPlace and collection slot return summaries used to store raw PlaceProjection suffixes. When such a suffix contained `ResourceOffset::Symbolic` or `ScaledSymbolic`, the embedded `Place` could remain callee-local and be replayed against caller state.

## 影響

Symbolic collection slot traversal proofs cannot be replayed soundly across function summaries. A caller may receive caller_storage[callee_i * stride], and range facts such as caller_i < len are not matched against the stale callee_i operand. This blocks generic source-derived proof replay and would otherwise encourage stdlib helper allowlists or marker-only proofs.

## 修正方針

Fixed by redesigning collection slot summary suffixes as typed summary projections. Every operand inside `ResourceOffset` is now either parameter-relative or the summary is not emitted. Shared recursive summarize / instantiate / translate helpers are used by normal summary replay, return transfer, return slot, and wrapper composition.

## 検証

- `cargo test -p nepl-core --lib collection_slot_summary_target -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot_summary_build_ops -- --test-threads=1`
- `cargo check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
