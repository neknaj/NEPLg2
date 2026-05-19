---
id: ISS-20260519T170102229Z-COLLECTION-POP-OWNER-ACCESSORS-LACK--B71B4C03
title: "collection pop owner accessors lack generic Copy-only policy"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "nodesrc/test_stdlib_collection_cleanup_contract.js, stdlib/alloc/collections/**, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260519T170102229Z-COLLECTION-POP-OWNER-ACCESSORS-LACK--B71B4C03: collection pop owner accessors lack generic Copy-only policy

## 概要

Current Copy-only collection safety relies on per-module policy checks for Pop result owner accessors. A new generic accessor that consumes FooPop<T> and returns Foo<T> could discard the item payload without Copy bound before OwnedBuffer initialized-cell drop traversal exists.

## 対象

- `nodesrc/test_stdlib_collection_cleanup_contract.js, stdlib/alloc/collections/**, tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `VecPop<T>`、`StackPop<T>`、`QueuePop<T>`、`DequePop<T>`、`RingBufferPop<T>`、`BinaryHeapPop<T>` は、更新後 collection owner と取り出した `Option<T>` payload を同時に持つ。
- `*_pop_*` owner accessor が result を値渡しで消費し、collection owner だけを返す場合、返されない `Option<T>` payload は同じ API 境界で閉じられる。
- 既存実装は個別 policy で Copy-only 境界を監視していたが、横断 policy は `Error<...>` owner accessor 専用で、`Pop<...>` owner accessor を構造的に検出していなかった。

## 問題

Current Copy-only collection safety relies on per-module policy checks for Pop result owner accessors. A new generic accessor that consumes FooPop<T> and returns Foo<T> could discard the item payload without Copy bound before OwnedBuffer initialized-cell drop traversal exists.

## 影響

A future collection pop/remove result can reintroduce non-Copy payload loss through an owner-returning accessor while RV-STDLIB-004 remains open.

## 修正方針

Extend the cross-collection source policy to detect generic owner-returning Pop result accessors by function type shape and require every generic parameter to carry Copy until drop traversal exists.

## 検証

Run the collection cleanup source policy, issue validation, and source-policy regression runner in warn-only mode.

## 対応

- `nodesrc/test_stdlib_collection_cleanup_contract.js` に pop-result owner accessor の横断検出を追加した。
- 判定は collection 名の個別 allowlist ではなく、関数型 `<(XPop<...>)->Owner<...>>`、値渡し parameter、generic return の形から行う。
- 現行の `vec_pop_vec` / `stack_pop_stack` / `queue_pop_queue` / `deque_pop_deque` / `ringbuffer_pop_buffer` / `binary_heap_pop_heap` を必ず検査対象に含める assertion を追加した。
- `Pop<T>` result から owner だけを返す accessor は、`OwnedBuffer<T>` / initialized prefix / moved slot / drop traversal が完成するまで generic parameter に `Copy` bound を要求する。

## 検証結果

- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: pass
- `node nodesrc/issues.js check`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
