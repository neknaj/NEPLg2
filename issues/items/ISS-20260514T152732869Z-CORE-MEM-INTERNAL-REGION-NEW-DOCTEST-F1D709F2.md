---
id: ISS-20260514T152732869Z-CORE-MEM-INTERNAL-REGION-NEW-DOCTEST-F1D709F2
title: "core/mem internal region_new doctest demonstrates forged owner token"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/core/mem/internal.nepl, nodesrc/test_stdlib_mem_internal_region_new_docs.js"
---

# ISS-20260514T152732869Z-CORE-MEM-INTERNAL-REGION-NEW-DOCTEST-F1D709F2: core/mem internal region_new doctest demonstrates forged owner token

## 概要

core/mem/internal.nepl documents region_new with a doctest that builds RegionToken<u8> from mem_ptr_wrap 128 and only reads region_size. That example does not prove allocator-issued ownership and can normalize the exact forged-token pattern that Resource IR rejects in user source.

## 対象

- `stdlib/core/mem/internal.nepl, nodesrc/test_stdlib_mem_internal_region_new_docs.js`

## 根拠

- `stdlib/core/mem/internal.nepl` の `region_new` doctest は、`RegionToken<u8>` を `mem_ptr_wrap 128` から構築し、`region_size` を読むだけで成功していた。
- `tests/stdlib/memory_safety.n.md` には fixed raw address / `str_addr` / borrowed `region_ptr` 由来 token を `dealloc_region` へ渡す compile_fail regression があり、compiler 側は forged owner token を拒否する設計になっている。
- canonical internal doctest が forged token を正常例として示すと、Stage 6 の `MemPtr = non-owning pointer` / `RegionToken = free-obligation owner wrapper` という説明と衝突する。

## 問題

core/mem/internal.nepl documents region_new with a doctest that builds RegionToken<u8> from mem_ptr_wrap 128 and only reads region_size. That example does not prove allocator-issued ownership and can normalize the exact forged-token pattern that Resource IR rejects in user source.

## 影響

The Stage 6 memory model says RegionToken is a free-obligation owner wrapper and MemPtr is only a non-owning view. A canonical internal doctest that succeeds on a fixed raw address undermines the documentation and regression signal for that separation.

## 修正方針

Rewrite the region_new doctest to derive the pointer from alloc_ptr, consume the resulting RegionToken through dealloc_region, and add a source-policy regression that prevents the internal region_new doctest from returning to fixed-address mem_ptr_wrap examples.

## 検証

Run the new source policy, the focused stdlib/core/mem/internal doctest, issue check, and diff whitespace check.

## 2026-05-15 Agent 1 修正

`region_new` の doctest を、固定 raw address から owner token を構築する例ではなく、`alloc_ptr<u8>` が返した allocator-issued pointer から `RegionToken<u8>` を作り、最後に `dealloc_region<u8>` で owner obligation を閉じる例へ差し替えた。

あわせて `nodesrc/test_stdlib_mem_internal_region_new_docs.js` を追加し、`region_new` doctest が `alloc_ptr` 由来 pointer と `dealloc_region` consumption を示すこと、かつ `region_new mem_ptr_wrap` / non-zero fixed raw address wrapping を正常例へ戻さないことを監視する。source policy runner にもこの検査を登録した。

検証:

- `node nodesrc/test_stdlib_mem_internal_region_new_docs.js`: passed
- `node nodesrc/tests.js -i stdlib/core/mem/internal.nepl --no-tree -o tmp/agent1-mem-internal-region-new-docs.json -j 1 --dist web/dist`: total=4, passed=4
