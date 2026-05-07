---
id: ISS-20260507T023409425Z-RESOURCE-IR-MISSES-DIRECT-MEMPTR-STO-8DEA4710
title: "Resource IR misses direct MemPtr store initialization after RegionToken projection"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary_variant_build.rs, nepl-core/src/resource/initialized_summary_cells.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260507T023409425Z-RESOURCE-IR-MISSES-DIRECT-MEMPTR-STO-8DEA4710: Resource IR misses direct MemPtr store initialization after RegionToken projection

## 概要

A RegionToken-derived MemPtr can be stored through direct store_i32 p and then loaded through direct load_i32 p, but the initialized-cell summary does not carry the Result::Ok-gated param cell back to p.raw.deref. The zero-offset mem_ptr_add path works, which shows the direct MemPtr projection and variant param-cell summary are inconsistent.

## 対象

- `nepl-core/src/resource/initialized_summary_variant_build.rs, nepl-core/src/resource/initialized_summary_cells.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

A RegionToken-derived MemPtr can be stored through direct store_i32 p and then loaded through direct load_i32 p, but the initialized-cell summary does not carry the Result::Ok-gated param cell back to p.raw.deref. The zero-offset mem_ptr_add path works, which shows the direct MemPtr projection and variant param-cell summary are inconsistent.

## 影響

Fixtures and stdlib code are pushed toward artificial mem_ptr_add p 0 projections, while direct MemPtr access can report resource.cell.uninit even after a checked store. This is a Resource IR precision bug and should not be hidden by weakening RawMemoryLoadCell strictness.

## 修正方針

Audit Result::Ok-gated raw cell initialization summaries for MemPtr parameters and RegionToken-derived projections. Ensure direct MemPtr raw field projections and zero-offset projected MemPtr paths produce the same typed cell fact, without treating ordinary i32 values as raw pointer proofs.

## 検証

Add a Resource IR regression for alloc_region/region_ptr, unwrap_ok store_i32 p value, then load_i32 p. Keep the existing mem_ptr_add zero-offset path passing and RawMemoryLoadCell strictness unchanged.
