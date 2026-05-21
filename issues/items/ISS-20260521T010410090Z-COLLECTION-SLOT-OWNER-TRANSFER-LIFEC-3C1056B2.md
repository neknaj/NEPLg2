---
id: ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2
title: "Collection slot owner-transfer lifecycle lacks payload value-flow proof"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/lower_collection_slot.rs, nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/typecheck/prefix_check.rs"
---

# ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2: Collection slot owner-transfer lifecycle lacks payload value-flow proof

## 概要

CollectionSlotLifecycle primitives such as InitializeEmpty, MoveOut, and ReplaceReturnOld change initialized/moved/replaced slot state using only storage anchor, offset, and type arguments. They do not carry typed payload source or destination evidence, so Resource IR can assert an owner transfer without proving that a payload value was consumed, returned, or materialized.

## 対象

- `nepl-core/src/resource/lower_collection_slot.rs, nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/typecheck/prefix_check.rs`

## 根拠

- 未記入

## 問題

CollectionSlotLifecycle primitives such as InitializeEmpty, MoveOut, and ReplaceReturnOld change initialized/moved/replaced slot state using only storage anchor, offset, and type arguments. They do not carry typed payload source or destination evidence, so Resource IR can assert an owner transfer without proving that a payload value was consumed, returned, or materialized.

## 影響

Future non-Copy collection APIs could satisfy slot lifecycle state transitions with annotations while the actual owner value flow is missing or mismatched, weakening memory-safety and type-safety guarantees.

## 修正方針

Redesign collection slot owner-transfer lifecycle events so initialize, move-out, and replace-return-old carry typed value-flow evidence or are lowered from a generic Resource IR proof that links the slot state transition to actual payload consume/materialization. Keep the droppable drop guard until real slot-drop elaboration exists.

## 検証

Add regressions for initialize without payload proof, non-Copy payload consumed on initialize, move-out materializing an output owner, replace-return-old returning old owner and consuming new owner, wrong payload type rejection, and continued replace-drop-old droppable rejection.
