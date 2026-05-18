---
id: ISS-20260518T222113382Z-COLLECTION-OWNER-RETURNING-ERROR-ACC-F869A5C5
title: "collection owner-returning error accessor policy is Vec-specific"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "nodesrc/test_stdlib_collection_cleanup_contract.js, nodesrc/run_source_policy_regressions.js, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md"
---

# ISS-20260518T222113382Z-COLLECTION-OWNER-RETURNING-ERROR-ACC-F869A5C5: collection owner-returning error accessor policy is Vec-specific

## 概要

After tightening Vec error owner accessors, the generic collection cleanup policy still only scans cleanup/free signatures. Owner-returning error accessors that recover generic collection owners are guarded by family-specific tests, so a new collection can reintroduce unconstrained recovery without a shared regression.

## 対象

- `nodesrc/test_stdlib_collection_cleanup_contract.js, nodesrc/run_source_policy_regressions.js, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md`

## 根拠

- `nodesrc/test_stdlib_collection_cleanup_contract.js` は generic `free` / `clear` / cleanup signature だけを横断検査しており、error payload から collection owner を返す accessor は Vec 専用 policy と個別 doctest に依存していた。
- `VecPushError<T>` / `VecTransformError<T>` / `VecSortMergeError<T>` の修正で同種の owner recovery surface が見つかったため、Stack / Queue / Deque / RingBuffer / BinaryHeap / List / BTreeMap / BTreeSet / HashMap / HashSet の accessor も同じ契約で監視する必要がある。

## 問題

After tightening Vec error owner accessors, the generic collection cleanup policy still only scans cleanup/free signatures. Owner-returning error accessors that recover generic collection owners are guarded by family-specific tests, so a new collection can reintroduce unconstrained recovery without a shared regression.

## 影響

The Copy-only boundary before OwnedBuffer initialized-cell drop traversal would rely on per-module policy additions and could miss future non-Copy owner recovery surfaces.

## 修正方針

Extend the collection cleanup contract policy to scan generic error owner accessors returning collection owners and require every generic parameter to carry Copy until drop traversal exists.

## 修正内容

- `stdlib/alloc/collections/**/*.nepl` の関数シグネチャから、`Error<...>` 値を消費して generic owner を返す error recovery accessor を検出する policy を追加した。
- 判定は Vec / HashMap などの個別型名リストではなく、関数型 `<(XError<...>)->Owner<...>>` と値渡し parameter / generic return の形から行う。
- `vec_realloc_region_error_region<T>` を含め、現行の generic owner recovery surface が `.T` / `.K` / `.V` / `.H` など全 generic parameter に `Copy` bound を持つことを横断検査するようにした。

## 検証

- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
