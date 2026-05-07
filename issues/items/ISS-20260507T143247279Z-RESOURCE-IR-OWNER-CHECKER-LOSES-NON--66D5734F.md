---
id: ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F
title: "Resource IR owner checker loses non-owning region_ptr_at Ok payload provenance"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_resource.rs, nepl-core/src/resource/place_utils.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F: Resource IR owner checker loses non-owning region_ptr_at Ok payload provenance

## 概要

region_ptr_at returns a bounds-checked MemPtr through Result::Ok, but the pointer is a non-owning projection of a borrowed RegionToken. Resource IR marked direct region_ptr as non-owning, yet the region_ptr_at implementation obtains the pointer through region_token_ptr_ref / mem_ptr_addr / mem_ptr_wrap and the non-owning raw view fact is lost before owner summary construction. A caller can match the Ok payload, wrap it with region_new, and pass the forged RegionToken to dealloc_region without resource.owner.no_free_obligation.

## 対象

- `nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_resource.rs, nepl-core/src/resource/place_utils.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- 未記入

## 問題

region_ptr_at returns a bounds-checked MemPtr through Result::Ok, but the pointer is a non-owning projection of a borrowed RegionToken. Resource IR marked direct region_ptr as non-owning, yet the region_ptr_at implementation obtains the pointer through region_token_ptr_ref / mem_ptr_addr / mem_ptr_wrap and the non-owning raw view fact is lost before owner summary construction. A caller can match the Ok payload, wrap it with region_new, and pass the forged RegionToken to dealloc_region without resource.owner.no_free_obligation.

## 影響

Safe source can turn a borrowed RegionToken projection into a forged RegionToken owner and free storage without holding the original free obligation. This violates the MemPtr = non-owning pointer / RegionToken = owner obligation separation required by the static-check complexity reduction plan.

## 修正方針

Treat compiler-provided borrowed RegionToken pointer references as non-owning raw address views in Resource IR lowering, so the view fact propagates through deref, mem_ptr_addr, mem_ptr_wrap, Result::Ok payload summaries, and helper-call owner consumption. Keep the lowering coverage gate aligned with the explicit view metadata, and add Rust and n.md compile_fail regressions for region_ptr_at Ok payload forging.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_at_ok_payload -- --nocapture; node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-region-ptr-at-forge.json -j 1 --dist web/dist

## 対応結果

2026-05-07 に修正済み。

- `region_ptr_at` の bounds-checked `Result::Ok(MemPtr<U>)` payload を Resource IR lowering 時点で `RawAddressViewKind::NonOwningProjection` として明示するようにした。
- `region_token_ptr_ref` が返す borrowed `&MemPtr<T>` の raw field も non-owning projection として扱い、`*region_token_ptr_ref` / `mem_ptr_addr` / `mem_ptr_wrap` を経由しても free obligation owner へ昇格しないようにした。
- Resource IR coverage gate は `region_ptr_at` / `region_token_ptr_ref` の reference projection を HIR 側でも数え、`RawAddressView` の target は alias metadata として扱うようにした。これにより coverage を弱めず、追加 projection の意味だけを揃えた。
- Rust regression と `.n.md` compile_fail regression で、`region_ptr_at` の Ok payload を `region_new` へ詰め直して `dealloc_region` する経路が `resource.owner.no_free_obligation` で拒否されることを固定した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
