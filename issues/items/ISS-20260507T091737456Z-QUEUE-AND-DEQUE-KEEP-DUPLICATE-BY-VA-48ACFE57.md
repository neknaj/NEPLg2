---
id: ISS-20260507T091737456Z-QUEUE-AND-DEQUE-KEEP-DUPLICATE-BY-VA-48ACFE57
title: "Queue and Deque keep duplicate by-value and *_ref observer APIs"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, stdlib/tests/queue.n.md, stdlib/tests/deque.n.md, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md, tests/stdlib/pipe_collections.n.md, nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js"
---

# ISS-20260507T091737456Z-QUEUE-AND-DEQUE-KEEP-DUPLICATE-BY-VA-48ACFE57: Queue and Deque keep duplicate by-value and *_ref observer APIs

## 概要

Queue and Deque still expose read-only observers twice: len/is_empty/peek consume the owner while len_ref/is_empty_ref/peek_ref keep the owner. Deque has the same split for len/cap/is_empty/peek_front/peek_back. This preserves a backward-compatible surface from the older borrow checker workaround phase and makes callers choose between two names for the same read-only operation.

## 対象

- `stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, stdlib/tests/queue.n.md, stdlib/tests/deque.n.md, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md, tests/stdlib/pipe_collections.n.md, nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`

## 根拠

- `Queue` は `len` / `is_empty` / `peek` が `Queue<T>` を by-value で受け取り、`len_ref` / `is_empty_ref` / `peek_ref` が借用版として残っていた。
- `Deque` は `len` / `cap` / `is_empty` / `peek_front` / `peek_back` が `Deque<T>` を by-value で受け取り、`len_ref` / `cap_ref` / `is_empty_ref` / `peek_front_ref` / `peek_back_ref` が借用版として残っていた。
- `Vec` / `RingBuffer` / `BitSet` / `SparseSet` / `SegmentTree` では primary observer 名を borrowed receiver に統一し、重複 `*_ref` surface を source policy で拒否している。Queue/Deque だけ旧 compatibility surface が残っていた。
- `tests/stdlib/pipe_collections.n.md` の Queue case も `len<i32> q` と `q2 |> peek` で owner-to-observer usage を残していた。

## 問題

Queue and Deque still expose read-only observers twice: len/is_empty/peek consume the owner while len_ref/is_empty_ref/peek_ref keep the owner. Deque has the same split for len/cap/is_empty/peek_front/peek_back. This preserves a backward-compatible surface from the older borrow checker workaround phase and makes callers choose between two names for the same read-only operation.

## 影響

Selfhost collection code and examples can continue to move owners accidentally for read-only observation, and tests can keep unnatural duplicated setup. The API surface also no longer matches Vec, RingBuffer, BitSet, SparseSet, SegmentTree, and other collections that use the primary observer name for borrowed reads and reject duplicate *_ref surfaces.

## 修正方針

Make the primary Queue and Deque observer names borrow the owner, remove duplicate *_ref observer APIs, and update doctests and collection fixtures to borrow explicitly and free owners after observation. Extend source-policy coverage so by-value observers and *_ref aliases cannot be reintroduced.

## 検証

- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/queue.nepl -i stdlib/alloc/collections/deque.nepl -i stdlib/tests/queue.n.md -i stdlib/tests/deque.n.md -i tests/stdlib/queue_collections.n.md -i tests/stdlib/deque_collections.n.md -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/queue-deque-borrowed-primary-observers.json -j 1 --dist web/dist`: total=18, passed=18

## 対応結果

`Queue.len` / `Queue.is_empty` / `Queue.peek` を `&Queue<T>` receiver に変更し、`len_ref` / `is_empty_ref` / `peek_ref` を削除した。

`Deque.len` / `Deque.cap` / `Deque.is_empty` / `Deque.peek_front` / `Deque.peek_back` を `&Deque<T>` receiver に変更し、`len_ref` / `cap_ref` / `is_empty_ref` / `peek_front_ref` / `peek_back_ref` を削除した。

関連 doctest / `.n.md` fixture は primary observer 名へ移行し、観測後に owner を `free` または terminal API で閉じる形へ更新した。`nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` は by-value observer と `*_ref` surface の再導入を拒否するよう拡張した。
