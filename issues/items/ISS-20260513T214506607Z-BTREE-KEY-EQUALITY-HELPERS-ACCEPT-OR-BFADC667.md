---
id: ISS-20260513T214506607Z-BTREE-KEY-EQUALITY-HELPERS-ACCEPT-OR-BFADC667
title: "BTree key equality helpers accept Ord keys without Copy"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js"
---

# ISS-20260513T214506607Z-BTREE-KEY-EQUALITY-HELPERS-ACCEPT-OR-BFADC667: BTree key equality helpers accept Ord keys without Copy

## 概要

btreemap_key_eq and btreeset_key_eq call ord_lt twice on by-value keys but only require Ord, so a public helper can accept non-Copy owning keys before borrowed comparison and Resource IR initialized-cell movement are available.

## 対象

- `stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`

## 根拠

- `stdlib/alloc/collections/btreemap/search.nepl` の `btreemap_key_eq` は、`ord_lt a b` と `ord_lt b a` の 2 回の by-value 比較で key equality を構成していた。
- `stdlib/alloc/collections/btreeset/search.nepl` の `btreeset_key_eq` も同じ構造であり、以前は `Ord` だけを要求していた。
- `core/traits/ord` の `ord_lt` は by-value な `(T, T) -> bool` なので、non-Copy key をこの helper に通すと同じ値を二度渡す境界になる。

## 問題

btreemap_key_eq and btreeset_key_eq call ord_lt twice on by-value keys but only require Ord, so a public helper can accept non-Copy owning keys before borrowed comparison and Resource IR initialized-cell movement are available.

## 影響

A non-Copy key can be consumed more than once through the equality helper boundary, weakening the Stage 6 Copy-only collection safety fence and hiding an ownership bug behind helper reuse.

## 修正方針

Constrain the equality helpers to Ord&Copy until borrowed key comparison and OwnedBuffer/InitializedCell based non-Copy collections are implemented, and add a source policy regression so the bound cannot regress.

## 検証

Run the BTree source policy regression and issue index/check.

## 対応結果

- `btreemap_key_eq` を `.K: Ord&Copy` に限定した。
- `btreeset_key_eq` を `.T: Ord&Copy` に限定した。
- `nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js` に、BTree key equality helper が `Ord&Copy` であり、`Ord` のみへ戻らないことを検査する source policy を追加した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)
