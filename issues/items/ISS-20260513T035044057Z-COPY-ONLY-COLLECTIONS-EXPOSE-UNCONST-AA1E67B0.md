---
id: ISS-20260513T035044057Z-COPY-ONLY-COLLECTIONS-EXPOSE-UNCONST-AA1E67B0
title: "Copy-only collections expose unconstrained cleanup APIs"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/{stack,queue,deque,ringbuffer,binary_heap,btreeset,btreemap,list}/**"
---

# ISS-20260513T035044057Z-COPY-ONLY-COLLECTIONS-EXPOSE-UNCONST-AA1E67B0: Copy-only collections expose unconstrained cleanup APIs

## 概要

Derived collection APIs such as Stack, Queue, Deque, RingBuffer, BinaryHeap, BTreeSet, BTreeMap, and List already require Copy payloads for construction, mutation, and observation, but their cleanup/free storage boundaries still accept unconstrained type parameters. This makes the current Copy-only implementation look safe for non-Copy payload owners even though element drop traversal and OwnedBuffer-based initialized-cell tracking are not implemented yet.

## 対象

- `stdlib/alloc/collections/{stack,queue,deque,ringbuffer,binary_heap,btreeset,btreemap,list}/**`

## 根拠

- `Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` は constructor / mutation / observer が `.T: Copy` 前提なのに、`free<T>` だけが unconstrained だった。
- `BTreeSet` / `BTreeMap` は storage allocation / read / mutation が Copy-only なのに、`free_storage` と public `free` が unconstrained だった。
- `List` は `push` / `tail` / `reverse` / observer が Copy-only なのに、`new<T>` / `free<T>` / `list_free_items<T>` が unconstrained だった。
- これらはいずれも `Vec<Option<T>>` または `Vec<T>` の storage-only dealloc に委譲しており、non-Copy payload の element drop traversal を持っていない。

## 問題

Derived collection APIs such as Stack, Queue, Deque, RingBuffer, BinaryHeap, BTreeSet, BTreeMap, and List already require Copy payloads for construction, mutation, and observation, but their cleanup/free storage boundaries still accept unconstrained type parameters. This makes the current Copy-only implementation look safe for non-Copy payload owners even though element drop traversal and OwnedBuffer-based initialized-cell tracking are not implemented yet.

## 影響

Callers can form or forge collection values whose free path is type-accepted without proving that element payloads are Copy or dropped. That weakens the collection cleanup contract tracked by RV-STDLIB-004 and hides the remaining OwnedBuffer redesign work behind generic signatures.

## 修正方針

Align cleanup/free and storage-only deallocation helpers with the existing Copy-only public contract. Keep non-Copy element support open under RV-STDLIB-004 and the OwnedBuffer Stage D design instead of pretending it is implemented.

## 検証

Add compile-fail regression coverage that a non-Copy payload collection cannot call free through these Copy-only collection APIs, then run the focused stdlib documentation/collection tests and issue validation.

## 修正結果

- Copy-only collection の public cleanup/free 境界を `.T: Copy` / `.K: Copy,.V: Copy` に揃えた。
- `BTreeSet` / `BTreeMap` の storage-only cleanup helper も同じ Copy bound に揃え、public API と internal helper の契約差をなくした。
- `List.new<T>` も `.T: Copy` に揃え、`List<T>` が non-Copy payload collection として構築できるように見える入口を閉じた。
- binary_heap focused doctest で `api/observer.nepl` が `eq` を module-local に import していない問題が見えたため、observer module 自身が `core/math` を import するようにした。これにより submodule が root facade の private import に依存しない。
- `tests/stdlib/collection_cleanup_contract.n.md` を追加し、non-Copy payload に対する Stack / Queue / Deque / RingBuffer / BTreeSet / BTreeMap / List の `free` 呼び出しが `type.trait_bound.unsatisfied` で拒否されることを固定した。
- source policy 側も、Stack / Queue / Deque / RingBuffer / BinaryHeap / BTreeSet / BTreeMap / List の cleanup 契約が Copy-only のまま保たれるように更新した。

## 残件

この issue は現行 Copy-only 実装の型契約漏れを塞ぐものであり、`RV-STDLIB-004` 全体の完了ではない。non-Copy payload collection は `OwnedBuffer<T>`、initialized prefix、move-out / replace / drop traversal、fallible update で owner を返す API へ再設計する必要がある。

## 検証結果

- `node nodesrc\test_stdlib_binary_heap_no_unsafe_unwraps.js && node nodesrc\test_stdlib_stack_no_unsafe_unwraps.js && node nodesrc\test_stdlib_queue_deque_no_unsafe_unwraps.js && node nodesrc\test_stdlib_ringbuffer_no_unsafe_unwraps.js && node nodesrc\test_stdlib_list_no_unsafe_unwraps.js && node nodesrc\test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`
- `node nodesrc\run_doctest.js -i tests\stdlib\collection_cleanup_contract.n.md -n 1`
- `node nodesrc\tests.js -i tests\stdlib\collection_cleanup_contract.n.md -i tests\stdlib\stack_collections.n.md -i tests\stdlib\queue_collections.n.md -i tests\stdlib\deque_collections.n.md -i tests\stdlib\ringbuffer_collections.n.md -i tests\stdlib\binary_heap_collections.n.md -i tests\stdlib\list_collections.n.md --no-tree -o tmp\agent1-collection-cleanup-contract-linear.json -j 4`
- `node nodesrc\tests.js -i tests\stdlib\pipe_collections.n.md -i stdlib\tests\stack.n.md -i stdlib\tests\queue.n.md -i stdlib\tests\deque.n.md -i stdlib\tests\ringbuffer.n.md -i stdlib\tests\binary_heap.n.md -i stdlib\tests\list.n.md --no-tree -o tmp\agent1-collection-cleanup-contract-stdlib.json -j 4`
- `node nodesrc\tests.js -i tests\stdlib\pipe_collections.n.md -i stdlib\tests\btreemap.n.md -i stdlib\tests\btreeset.n.md --no-tree -o tmp\agent1-collection-cleanup-contract-btree.json -j 4`
