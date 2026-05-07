---
id: ISS-20260507T092629042Z-BINARYHEAP-KEEPS-DUPLICATE-BY-VALUE--31161123
title: "BinaryHeap keeps duplicate by-value and *_ref observer APIs"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/collections/binary_heap.nepl, stdlib/tests/binary_heap.n.md, tests/stdlib/binary_heap_collections.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js"
---

# ISS-20260507T092629042Z-BINARYHEAP-KEEPS-DUPLICATE-BY-VALUE--31161123: BinaryHeap keeps duplicate by-value and *_ref observer APIs

## 概要

BinaryHeap still exposes read-only observers twice: len/cap/is_empty/peek consume the owner while len_ref/cap_ref/is_empty_ref/peek_ref borrow it. This keeps an old compatibility surface and allows read-only observation to move owners.

## 対象

- `stdlib/alloc/collections/binary_heap.nepl, stdlib/tests/binary_heap.n.md, tests/stdlib/binary_heap_collections.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

BinaryHeap still exposes read-only observers twice: len/cap/is_empty/peek consume the owner while len_ref/cap_ref/is_empty_ref/peek_ref borrow it. This keeps an old compatibility surface and allows read-only observation to move owners.

## 影響

Selfhost priority queue usage can accidentally consume a heap for length or peek checks, and doctests can keep duplicate setup instead of one owner workflow. The API also diverges from collections that now use primary observer names for borrowed reads.

## 修正方針

Make BinaryHeap primary observer names borrow the owner, remove duplicate *_ref observers, update tests and source-policy so by-value observers cannot return.

## 検証

node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js; node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/binary-heap-primary-borrowed-observers.json -j 1 --dist web/dist
