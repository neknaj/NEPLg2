---
id: ISS-20260507T092629042Z-BINARYHEAP-KEEPS-DUPLICATE-BY-VALUE--31161123
title: "BinaryHeap keeps duplicate by-value and *_ref observer APIs"
area: stdlib
status: fixed
resolved: true
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

- `BinaryHeap.len` / `cap` / `is_empty` / `peek` が owner を値で受け取り、内部で `free` していた。
- 同じ読み取り機能を `len_ref` / `cap_ref` / `is_empty_ref` / `peek_ref` が借用版として重複提供していた。
- `stdlib/tests/binary_heap.n.md` と `tests/stdlib/binary_heap_collections.n.md` に `len_ref` / `peek_ref` と by-value observer 呼び出しが残り、API 利用側にも古い owner 消費モデルが漏れていた。

## 問題

BinaryHeap still exposes read-only observers twice: len/cap/is_empty/peek consume the owner while len_ref/cap_ref/is_empty_ref/peek_ref borrow it. This keeps an old compatibility surface and allows read-only observation to move owners.

## 影響

Selfhost priority queue usage can accidentally consume a heap for length or peek checks, and doctests can keep duplicate setup instead of one owner workflow. The API also diverges from collections that now use primary observer names for borrowed reads.

## 修正方針

Make BinaryHeap primary observer names borrow the owner, remove duplicate *_ref observers, update tests and source-policy so by-value observers cannot return.

## 検証

- `node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/binary-heap-primary-borrowed-observers.json -j 1 --dist web/dist`: total=14, passed=14

## 対応結果

- `BinaryHeap.len` / `cap` / `is_empty` / `peek` を primary borrowed observer API に変更した。
- `len_ref` / `cap_ref` / `is_empty_ref` / `peek_ref` を削除し、互換用の重複 surface を閉じた。
- BinaryHeap doctest と collection integration test を明示 borrow と `free` に更新した。
- `nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js` に by-value observer と `*_ref` 再導入を拒否する regression を追加した。
