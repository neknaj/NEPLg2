---
id: ISS-20260517T031453210Z-REGIONTOKEN-RAW-IDENTITY-REFERENCE-R-BB2D917B
title: "RegionToken raw identity reference remains on public mem facade"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "stdlib/core/mem/types.nepl; stdlib/core/mem/internal.nepl; stdlib/core/mem/pointer/region.nepl; tests/stdlib/memory_safety.n.md; nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260517T031453210Z-REGIONTOKEN-RAW-IDENTITY-REFERENCE-R-BB2D917B: RegionToken raw identity reference remains on public mem facade

## 概要

region_token_raw_ref is defined as a public helper in mem/types and is re-exported by the safe core/mem facade, so ordinary source can obtain a reference to RegionToken.raw even though direct RegionToken field projection is restricted.

## 対象

- `stdlib/core/mem/types.nepl; stdlib/core/mem/internal.nepl; stdlib/core/mem/pointer/region.nepl; tests/stdlib/memory_safety.n.md; nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- `stdlib/core/mem/types.nepl` は safe `core/mem` facade から public re-export される。
- `region_token_raw_ref<T>(&RegionToken<T>) -> &i32` が `mem/types` にあると、通常 source が `*region_token_raw_ref &region` で free-obligation raw identity を読める。
- 既に `RegionToken.raw` の direct field projection は `type.owner_token.field_access_restricted` で拒否しているため、同じ identity を public helper で読める状態は境界設計として不整合である。

## 問題

region_token_raw_ref is defined as a public helper in mem/types and is re-exported by the safe core/mem facade, so ordinary source can obtain a reference to RegionToken.raw even though direct RegionToken field projection is restricted.

## 影響

The Stage 6 MemPtr non-owning / RegionToken owner split still exposes the raw free-obligation identity through a safe facade helper, weakening the boundary between public observation and raw-memory implementation proof.

## 修正方針

Move region_token_raw_ref to mem/internal, keep public region_size/region_in_bounds metadata observers in mem/types, update RegionToken projection users to the internal helper, and add policy/compile-fail coverage so safe core/mem imports cannot access raw owner identity.

## 検証

Run core/mem source policy, focused memory_safety compile-fail doctest, and focused region pointer doctest.

## 解決内容

- `region_token_raw_ref` を `stdlib/core/mem/types.nepl` から削除し、`stdlib/core/mem/internal.nepl` の raw memory boundary helper に移した。
- `region_ptr` / `region_ptr_at` は従来どおり internal helper 経由で raw identity を借用するが、safe `core/mem` facade からは raw identity accessor が見えない。
- `region_token_size_ref` / `region_size` / `region_in_bounds` は metadata observer として `mem/types` に残し、raw free-obligation identity とサイズ観測を分けた。
- `nodesrc/test_stdlib_core_mem_boundary.js` に、`mem/types` が `region_token_raw_ref` を公開しないこと、`mem/internal` が所有することを固定した。
- `tests/stdlib/memory_safety.n.md` に、safe `core/mem` import では `region_token_raw_ref` が未定義になる compile-fail regression を追加した。

## 検証結果

- `node nodesrc/test_stdlib_core_mem_boundary.js`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 30`: pass
- `node nodesrc/run_doctest.js -i stdlib/core/mem/pointer/region.nepl -n 1`: pass
