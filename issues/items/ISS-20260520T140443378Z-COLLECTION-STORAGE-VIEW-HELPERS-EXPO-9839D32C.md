---
id: ISS-20260520T140443378Z-COLLECTION-STORAGE-VIEW-HELPERS-EXPO-9839D32C
title: "Collection storage view helpers expose non-Copy payload internals"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/{hashmap,hashset,btreemap,btreeset}/storage.nepl, nodesrc/test_stdlib_collection_cleanup_contract.js, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260520T140443378Z-COLLECTION-STORAGE-VIEW-HELPERS-EXPO-9839D32C: Collection storage view helpers expose non-Copy payload internals

## 概要

HashMap/HashSet/BTreeMap/BTreeSet storage helper modules can return borrowed Vec<Option<T>> storage views for unconstrained generic payloads. Even though the view is borrowed, direct module imports expose internal payload storage shape before non-Copy collection drop traversal and initialized/moved cell proofs exist.

## 対象

- `stdlib/alloc/collections/{hashmap,hashset,btreemap,btreeset}/storage.nepl, nodesrc/test_stdlib_collection_cleanup_contract.js, tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `stdlib/alloc/collections/btreemap/storage.nepl` の `btreemap_storage_keys` / `btreemap_storage_values` が、`.K` / `.V` に `Copy` bound を要求せず `&Vec<Option<.K>>` / `&Vec<Option<.V>>` を返していた。
- `stdlib/alloc/collections/btreeset/storage.nepl` の `btreeset_storage_keys` が、`.T` に `Copy` bound を要求せず `&Vec<Option<.T>>` を返していた。
- `stdlib/alloc/collections/hashmap/storage.nepl` / `hashset/storage.nepl` も同じ形で payload-bearing storage view を返しており、direct import で内部 `Vec<Option<T>>` 形状を見せられる。
- これらは payload を値として move しないが、現行 collection は non-Copy payload の drop traversal / moved slot / initialized cell proof が未完成であり、内部 storage view surface も Copy-only 境界へ揃える必要がある。

## 問題

HashMap/HashSet/BTreeMap/BTreeSet storage helper modules can return borrowed Vec<Option<T>> storage views for unconstrained generic payloads. Even though the view is borrowed, direct module imports expose internal payload storage shape before non-Copy collection drop traversal and initialized/moved cell proofs exist.

## 影響

Unsupported non-Copy collection payloads can appear to have a safe storage-view surface, weakening the Copy-only boundary that protects collection cleanup until OwnedBuffer/InitializedCell/Resource IR drop traversal is complete.

## 修正方針

Require Copy on payload-bearing borrowed storage view helpers and add a structural source policy that rejects generic borrowed Vec<Option<T>> storage views without Copy. Add compile-fail doctests for direct storage module imports.

## 検証

Run collection cleanup source policy and focused collection_cleanup_contract doctests.

## 対応

- `btreemap_storage_keys<K,V>` / `hashmap_storage_keys<K,V>` / `btreeset_storage_keys<T>` / `hashset_storage_keys<T>` は payload key 側に `Copy` bound を要求するようにした。
- `btreemap_storage_values<K,V>` / `hashmap_storage_values<K,V>` は payload value 側に `Copy` bound を要求するようにした。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` に、`&Vec<Option<T>>` を返す payload-bearing borrowed storage view helper を関数型から横断検出し、該当 payload generic に `Copy` bound を要求する policy を追加した。
- `tests/stdlib/collection_cleanup_contract.n.md` に、direct storage module import から non-Copy payload storage view を得ようとする 6 件の compile-fail regression を追加した。
- `btreemap/storage.nepl` と `btreeset/storage.nepl` の新しい doc comment には、同じ Copy-only 境界を示す doc-test を追加し、documentation contract の baseline を悪化させないようにした。

## 検証結果

- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree --dist web/dist -o tmp/agent1-collection-storage-view-contract.json -j 4 --assert-io`: 43/43 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap/storage.nepl -i stdlib/alloc/collections/btreeset/storage.nepl --no-tree --dist web/dist -o tmp/agent1-btree-storage-view-docs.json -j 2 --assert-io`: 3/3 passed
- `node nodesrc/test_stdlib_hashmap_storage_contract.js`: passed
- `node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`: passed
- `node nodesrc/test_stdlib_documentation_contract.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
