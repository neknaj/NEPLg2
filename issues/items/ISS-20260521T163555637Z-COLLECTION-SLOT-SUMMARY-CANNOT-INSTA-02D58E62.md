---
id: ISS-20260521T163555637Z-COLLECTION-SLOT-SUMMARY-CANNOT-INSTA-02D58E62
title: "Collection slot summary cannot instantiate symbolic offset operands across calls"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_target.rs
---

# ISS-20260521T163555637Z-COLLECTION-SLOT-SUMMARY-CANNOT-INSTA-02D58E62: Collection slot summary cannot instantiate symbolic offset operands across calls

## 概要

CollectionSlotLifecycleSummaryPlace stores only a parameter index plus a raw PlaceProjection suffix. When the suffix contains PlaceProjection::StorageOffset(ResourceOffset::Symbolic or ScaledSymbolic), the embedded Place remains a callee-local operand. instantiate_summary_target attaches the suffix to the caller argument but does not recursively instantiate those embedded Places.

## 対象

- `nepl-core/src/resource/collection_slot_summary_target.rs`

## 根拠

- 未記入

## 問題

CollectionSlotLifecycleSummaryPlace stores only a parameter index plus a raw PlaceProjection suffix. When the suffix contains PlaceProjection::StorageOffset(ResourceOffset::Symbolic or ScaledSymbolic), the embedded Place remains a callee-local operand. instantiate_summary_target attaches the suffix to the caller argument but does not recursively instantiate those embedded Places.

## 影響

Symbolic collection slot traversal proofs cannot be replayed soundly across function summaries. A caller may receive caller_storage[callee_i * stride], and range facts such as caller_i < len are not matched against the stale callee_i operand. This blocks generic source-derived proof replay and would otherwise encourage stdlib helper allowlists or marker-only proofs.

## 修正方針

Redesign summary place representation so every operand inside ResourceOffset is parameter-relative or explicitly non-serializable. Add shared recursive summarize/instantiate helpers for PlaceProjection and ResourceOffset, then make DropTraversal summaries carry only caller-replayable symbolic operands and typed range evidence.

## 検証

Add regressions where helper(storage, i, len) certifies a symbolic slot traversal under 0 <= i && i < len, a wrapper forwards that summary, caller replay substitutes caller_i into ResourceOffset::ScaledSymbolic, and missing bound/stride mismatch still reports RangeProofRequired.
