---
id: ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A
title: "Vec stores backing storage as MemPtr owner field instead of RegionToken owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260514T085248522Z-VEC-STORES-BACKING-STORAGE-AS-MEMPTR-FFC9775A: Vec stores backing storage as MemPtr owner field instead of RegionToken owner

## 概要

Vec still stores backing storage as data <MemPtr<T>> plus a separate VecStorageState, leaving MemPtr as an owner-like public field after ByteBuf and ByteBuilder moved to RegionToken. This keeps Stage 6 dependent on a raw pointer storage exception and makes Resource IR treat a non-owning pointer shape as a free-obligation carrier.

## 対象

- `stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- `stdlib/alloc/collections/vec/types.nepl` の `Vec<T>` が `data <MemPtr<T>>` を public storage field として持っていた。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional allowlist に `stdlib/alloc/collections/vec/types.nepl::Vec.data::MemPtr<.T>` が残っていた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr = non-owning pointer` とし、free obligation owner を `RegionToken` / `OwnedBuffer` 系へ分離することを要求している。

## 問題

Vec still stores backing storage as data <MemPtr<T>> plus a separate VecStorageState, leaving MemPtr as an owner-like public field after ByteBuf and ByteBuilder moved to RegionToken. This keeps Stage 6 dependent on a raw pointer storage exception and makes Resource IR treat a non-owning pointer shape as a free-obligation carrier.

## 影響

The static-check complexity reduction cannot finish while Vec exposes a MemPtr storage field. Collection users and self-host code can continue to depend on transitional raw storage layout, delaying OwnedBuffer and initialized-cell based collection safety.

## 修正方針

Move Vec storage ownership to a RegionToken<T> field, derive MemPtr<T> only as a borrowed view from the token, update allocation/grow/free/read/write paths, and remove Vec.data from the MemPtr owner-field transitional baseline.

## 検証

Run focused Vec doctests and collection tests, source policies for MemPtr owner fields and Vec boundaries, ResourceIR owner regressions, issue index/check, trunk build when Rust-facing behavior is affected.

## 解決内容

`Vec<T>` の storage owner field を `data <MemPtr<T>>` から `region <RegionToken<T>>` へ移した。`data_mem_ptr<T>` / `vec_storage_mem_ptr<T>` / mutation / transform / sort は、`RegionToken<T>` 参照から non-owning `MemPtr<T>` view を作り、戻り値では `RegionToken<T>` owner を移す。

`alloc_region<T>` / `dealloc_region<T>` を Vec storage allocation / cleanup の owner boundary とし、grow 失敗時は `vec_realloc_region_or_free<T>` が旧 `RegionToken<T>` を消費して free obligation を閉じる。これにより `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional baseline から `Vec.data` を削除した。

この issue は `OwnedBuffer<T>` 完成ではない。`RegionToken<T>` はまだ forgeable であり、non-Copy payload の initialized prefix / move-out / drop traversal / owner-preserving fallible update は `RV-STDLIB-004` と Stage D の残件として継続する。

## 検証結果

- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: passed。transitional field は `RegionToken.ptr` の 1 件だけ。
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed。
- `node nodesrc/tests.js -i stdlib\alloc\collections\vec --no-tree -o tmp\agent1-vec-region-token-owner-doctests.json -j 1 --dist web/dist`: total=41, passed=41。
- `node nodesrc/tests.js -i tests\stdlib\vec_collections.n.md -i tests\stdlib\sort.n.md -i tests\stdlib\sort_simple.n.md -i tests\stdlib\capacity_stack.n.md -i tests\stdlib\collection_cleanup_contract.n.md --no-tree -o tmp\agent1-vec-region-token-owner-focused-tests.json -j 1 --dist web/dist`: total=33, passed=33。
