---
id: ISS-20260429T203138138Z-DEQUE-STILL-USES-RAW-HEADER-AND-RAW--5E76074E
title: "Deque still uses raw header and raw element storage"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/collections/deque.nepl, nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js, stdlib/tests/deque.n.md, tests/stdlib/deque_collections.n.md"
---

# ISS-20260429T203138138Z-DEQUE-STILL-USES-RAW-HEADER-AND-RAW--5E76074E: Deque still uses raw header and raw element storage

## 概要

Queue and RingBuffer have moved to typed Vec<Option<T>> storage, but Deque still owns a raw 16-byte header plus raw element buffer. The logical slot state is encoded by len/head arithmetic and uninitialized raw cells instead of an enum-like Option<T> value.

## 対象

- `stdlib/alloc/collections/deque.nepl, nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js, stdlib/tests/deque.n.md, tests/stdlib/deque_collections.n.md`

## 根拠

- `stdlib/alloc/collections/deque.nepl` は `Deque<T>` を `hdr <MemPtr<u8>>` だけで表し、`[len, cap, head, data_ptr]` を raw 16 byte header に保存していた。
- element storage は `alloc_ptr<T>` で確保した raw buffer で、live slot / inactive slot の状態は `len` と `head` の計算にのみ依存し、slot 自体は `Option<T>` などの型で初期化状態を持っていなかった。
- grow / pop / peek / free は `load<T>` / `store<T>` / `dealloc_raw` と `mem_ptr_addr` を直接使っており、Resource IR が Queue / RingBuffer と同じ typed storage invariant を利用できない状態だった。
- `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` も Deque だけ raw deallocation を許容する古い source policy を残していた。

## 問題

Queue and RingBuffer have moved to typed Vec<Option<T>> storage, but Deque still owns a raw 16-byte header plus raw element buffer. The logical slot state is encoded by len/head arithmetic and uninitialized raw cells instead of an enum-like Option<T> value.

## 影響

Deque remains outside the Resource IR friendly collection storage model. Self-host code that uses deque can still depend on raw header provenance and raw cell initialization instead of type-checked slot state.

## 修正方針

Migrate Deque to len/cap/head/items fields with Vec<Option<T>> storage, require Copy payloads until collection-wide drop traversal is designed, and update source policy/tests so raw header/data storage cannot return.

## 修正内容

- `Deque<T>` を `len/cap/head/items <Vec<Option<T>>>` を持つ owner struct に変更した。
- live slot は `Some(value)`、inactive slot は `None` として表現し、`with_capacity` は全 slot を `None` で初期化するようにした。
- `push_front` / `push_back` / grow / clear / peek / pop は `Vec<Option<T>>` の `get_ref` / `replace_ref` 経由で slot state を扱うようにした。
- `.T: Copy` を public mutating/read path に明示した。非 Copy payload の drop traversal は collection-wide cleanup issue に残す。
- `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` を更新し、Deque が raw header / raw element storage へ戻らないことを source policy で固定した。
- borrowed observation を使う `tests/stdlib/deque_collections.n.md` では、最後に `free` して owner を閉じるようにした。

## 検証

- `git diff --check`: passed
- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/deque_collections.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/tests.js -i stdlib/tests/deque.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/deque-typed-storage-focused.json -j 1 --dist web/dist`: `total=4`, `passed=4`
- `node nodesrc/tests.js -i stdlib/alloc/collections/deque.nepl --no-tree -o tmp/deque-typed-storage-docs.json -j 1 --dist web/dist`: `total=2`, `passed=2`
- `node nodesrc/issues.js check`: passed
