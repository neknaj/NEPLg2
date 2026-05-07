---
id: ISS-20260507T023409425Z-RESOURCE-IR-MISSES-DIRECT-MEMPTR-STO-8DEA4710
title: "Resource IR misses direct MemPtr store initialization after RegionToken projection"
area: core
status: fixed
resolved: true
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

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_applies_result_ok_region_ptr_direct_store_initialization -- --nocapture` で、`alloc_region` / `region_ptr` 由来の direct `MemPtr<i32>` に対する `store_i32 p` -> `load_i32 p` が `resource.cell.uninit` なしで通ることを確認した。
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_applies_result_ok_region_ptr_at_direct_store_initialization -- --nocapture` で、`region_ptr_at` の `Result::Ok` payload 由来の direct `MemPtr<i32>` でも同じ初期化 summary が適用されることを確認した。
- 現行 Resource IR は `RawAddressView` / raw-address return summary / variant-param initialized-cell summary により、direct `MemPtr` projection と zero-offset `mem_ptr_add` projection を同じ raw cell address として扱える。

## 問題

A RegionToken-derived MemPtr can be stored through direct store_i32 p and then loaded through direct load_i32 p, but the initialized-cell summary does not carry the Result::Ok-gated param cell back to p.raw.deref. The zero-offset mem_ptr_add path works, which shows the direct MemPtr projection and variant param-cell summary are inconsistent.

## 影響

Fixtures and stdlib code are pushed toward artificial mem_ptr_add p 0 projections, while direct MemPtr access can report resource.cell.uninit even after a checked store. This is a Resource IR precision bug and should not be hidden by weakening RawMemoryLoadCell strictness.

## 修正方針

Audit Result::Ok-gated raw cell initialization summaries for MemPtr parameters and RegionToken-derived projections. Ensure direct MemPtr raw field projections and zero-offset projected MemPtr paths produce the same typed cell fact, without treating ordinary i32 values as raw pointer proofs.

## 解決内容

調査時点の remote main では direct `region_ptr` / `region_ptr_at` 由来の `MemPtr` store-load 経路はすでに正しく扱われており、追加した最小回帰テストでも `RawMemoryLoadCell` の `Uninit` 診断は発生しなかった。

- `resource_ir_cell_check_applies_result_ok_region_ptr_direct_store_initialization` を追加し、`region_ptr &region` で得た `MemPtr<i32>` に対する direct `store_i32 p` -> direct `load_i32 p` を固定した。
- `resource_ir_cell_check_applies_result_ok_region_ptr_at_direct_store_initialization` を追加し、`region_ptr_at` の `Result::Ok p` payload でも variant-param initialized-cell summary が direct `p.raw.deref` へ適用されることを固定した。
- `RawMemoryLoadCell` の strictness は緩めていない。ordinary `i32` を raw pointer proof として扱う変更も入れていない。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_applies_result_ok_region_ptr_direct_store_initialization -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_applies_result_ok_region_ptr_at_direct_store_initialization -- --nocapture`: passed
