---
id: ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836
title: "Collection slot lifecycle intrinsic must not leak through public stdlib wrappers"
area: compiler
status: open
resolved: false
priority: P1
type: security
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/source_capability/**, nepl-core/src/resource_primitives/collection_slot.rs, stdlib/alloc/collections/**"
---

# ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836: Collection slot lifecycle intrinsic must not leak through public stdlib wrappers

## 概要

Collection slot lifecycle proof is now a typed Resource IR primitive, but the compiler does not yet statically prove that stdlib functions wrapping collection_slot_* intrinsics stay internal and cannot become public/re-exported safe APIs. A public wrapper could let user source indirectly emit lifecycle events without the intended compiler-owned proof boundary.

## 対象

- `nepl-core/src/source_capability/**, nepl-core/src/resource_primitives/collection_slot.rs, stdlib/alloc/collections/**`

## 根拠

- subagent review confirmed that `CollectionSlotLifecycleEvent` / `CollectionSlotLifecycleOp` / `CollectionSlotLifecyclePrimitive` are generic enum-backed proof boundaries rather than Vec/OwnedBuffer function allowlists.
- The remaining risk is the export surface: configured stdlib source may contain lifecycle intrinsics, but there is no dedicated compiler check that such functions cannot be made public or re-exported as ordinary safe APIs.
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) requires typed source capability boundaries to stay compiler-owned and not degrade into module/path convention.

## 問題

Collection slot lifecycle proof is now a typed Resource IR primitive, but the compiler does not yet statically prove that stdlib functions wrapping collection_slot_* intrinsics stay internal and cannot become public/re-exported safe APIs. A public wrapper could let user source indirectly emit lifecycle events without the intended compiler-owned proof boundary.

## 影響

Non-Copy collection payload support could gain a public escape hatch: user code would not call the intrinsic directly, but would still drive Initialize/MoveOut/Drop/StorageDealloc events through a public stdlib surface. That would undermine the generic proof boundary without obvious module allowlists.

## 修正方針

Add a generic source-capability/export-surface check that rejects or reports collection slot lifecycle intrinsic reachability from public stdlib exports unless the function is explicitly modeled as compiler-owned internal lowering. Keep this as an enum/match checked policy, not path-string allowlists.

## 検証

Add regressions for a stdlib public wrapper and re-export around collection_slot_* intrinsics; user source must not receive lifecycle authority through them, while compiler-owned internal lowering remains accepted.
