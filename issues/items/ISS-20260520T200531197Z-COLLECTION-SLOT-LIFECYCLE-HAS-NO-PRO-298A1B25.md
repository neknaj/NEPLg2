---
id: ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25
title: "Collection slot lifecycle has no production lowering producer"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/lower*.rs, nepl-core/src/resource/model.rs, stdlib/alloc/collections/**, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25: Collection slot lifecycle has no production lowering producer

## 概要

ResourceOp::CollectionSlotLifecycle is now checked by Resource IR, but no production lowering path emits it from real collection API semantics. Current regression coverage manually constructs the ResourceOp, so real Vec/OwnedBuffer operations cannot yet rely on the generic slot lifecycle proof.

## 対象

- `nepl-core/src/resource/lower*.rs, nepl-core/src/resource/model.rs, stdlib/alloc/collections/**, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

ResourceOp::CollectionSlotLifecycle is now checked by Resource IR, but no production lowering path emits it from real collection API semantics. Current regression coverage manually constructs the ResourceOp, so real Vec/OwnedBuffer operations cannot yet rely on the generic slot lifecycle proof.

## 影響

The compiler can prove slot lifecycle only for hand-written Resource IR. If stdlib non-Copy collection support proceeds before a typed lowering/annotation path exists, safety may fall back to module allowlists, inlining assumptions, or unchecked raw memory conventions.

## 修正方針

Design and implement a typed lowering source for collection slot lifecycle events. The producer must derive Initialize/BorrowRead/MoveOut/Replace/Drop/StorageDealloc events from source-level collection semantics or explicit compiler-owned annotations, not from stdlib function-name allowlists. It must feed ResourceOp::CollectionSlotLifecycle and preserve spans for diagnostics.

## 検証

cargo test -p nepl-core collection_slot_lowering -- --test-threads=1; cargo test -p nepl-core resource_ir_collection_slot -- --test-threads=1; node nodesrc/test_resource_checker_responsibility.js; node nodesrc/issues.js check --dir issues; git diff --check
