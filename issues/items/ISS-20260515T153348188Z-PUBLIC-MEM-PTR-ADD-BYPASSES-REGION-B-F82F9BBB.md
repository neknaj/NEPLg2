---
id: ISS-20260515T153348188Z-PUBLIC-MEM-PTR-ADD-BYPASSES-REGION-B-F82F9BBB
title: "public mem_ptr_add bypasses region bounds proof"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/effect_diagnostic.rs; nepl-core/src/resource/effect.rs; nepl-core/src/compiler.rs; tests/stdlib/memory_safety.n.md; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T153348188Z-PUBLIC-MEM-PTR-ADD-BYPASSES-REGION-B-F82F9BBB: public mem_ptr_add bypasses region bounds proof

## 概要

User source can call public mem_ptr_add on an allocator-derived RegionToken pointer, create an out-of-bounds MemPtr, and then pass it to checked load/store APIs. This bypasses region_ptr_at bounds and alignment proof.

## 対象

- `nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/effect.rs; nepl-core/src/compiler.rs; tests/stdlib/memory_safety.n.md`

## 根拠

- [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の `MemPtr = non-owning pointer` / raw-memory-backed public API migration 方針に反する。
- `alloc_region -> region_ptr -> mem_ptr_add -> store_i32` の経路では、`region_ptr_at` が持つ bounds / alignment proof を通らずに `MemPtr` が生成されていた。
- Resource IR lowering は `mem_ptr_add` を一般的な raw address arithmetic と同じ `RawAddressViewKind::Offset` として表現していたため、effect boundary が public `MemPtr` offset view を raw structural boundary operation として診断できなかった。

## 問題

User source can call public mem_ptr_add on an allocator-derived RegionToken pointer, create an out-of-bounds MemPtr, and then pass it to checked load/store APIs. This bypasses region_ptr_at bounds and alignment proof.

## 影響

Memory safety is not compiler-proven: a safe-looking MemPtr can be derived outside its OwnedRegion bounds without going through ResourceIR raw-memory boundary diagnostics.

## 修正方針

Introduce a dedicated RawAddressViewKind::MemPtrOffset for mem_ptr_add, treat it as a raw structural boundary operation in ResourceIR effect boundary checking, allow it only for compiler-owned raw-memory boundary sources, and add compile_fail regression coverage for the RegionToken/mem_ptr_add bypass.

## 検証

Focused compiler/resource tests and stdlib memory_safety doctest must reject user-level mem_ptr_add offset views with resource.raw.memory_outside_boundary while preserving stdlib raw boundary modules.

## 対応結果

- `ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary` を追加し、`RawAddressViewKind::MemPtrOffset` を raw structural boundary 外では `resource.raw.memory_outside_boundary` として拒否する。
- effect diagnostic model を `effect_diagnostic.rs` へ分離し、`effect.rs` は report construction に集中させた。
- `RawAddressViewKind::NonOwningProjection` は offset view と分けたままにし、検査済み projection と任意 pointer arithmetic を同じ扱いにしない。
- `tests/stdlib/memory_safety.n.md` に user source から `mem_ptr_add` で `RegionToken` bounds を迂回する compile_fail 回帰を追加した。
