---
id: ISS-20260507T085551696Z-PIPE-COLLECTIONS-RINGBUFFER-DOCTEST--5893794D
title: "pipe_collections RingBuffer doctest passes owner to borrow-only APIs"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: "tests/stdlib/pipe_collections.n.md, stdlib/alloc/collections/ringbuffer.nepl"
---

# ISS-20260507T085551696Z-PIPE-COLLECTIONS-RINGBUFFER-DOCTEST--5893794D: pipe_collections RingBuffer doctest passes owner to borrow-only APIs

## 概要

tests/stdlib/pipe_collections.n.md::doctest#7 still calls len<i32> rb and rb2 |> peek after RingBuffer len/peek are borrow-only APIs. The selected std/test import failure is fixed, but this stale fixture now fails with type.overload.no_match and hides RingBuffer pipe coverage.

## 対象

- `tests/stdlib/pipe_collections.n.md, stdlib/alloc/collections/ringbuffer.nepl`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-selective-import-rebase.json -j 1 --dist web/dist`: total=8, passed=7, failed=1。
- 残った `doctest#7` は `len<i32> rb` と `rb2 |> peek` で `type.overload.no_match` を出していた。
- `stdlib/alloc/collections/ringbuffer.nepl` の `len` / `peek` はどちらも `&RingBuffer<T>` を受け取る borrow-only observer API であり、owner を渡す fixture が古かった。
- 観測後の `rb` / `rb2` owner も解放されておらず、現在の ownership 方針に合っていなかった。

## 問題

tests/stdlib/pipe_collections.n.md::doctest#7 still calls len<i32> rb and rb2 |> peek after RingBuffer len/peek are borrow-only APIs. The selected std/test import failure is fixed, but this stale fixture now fails with type.overload.no_match and hides RingBuffer pipe coverage.

## 影響

The pipe_collections suite cannot fully validate collection pipe usage, and the failing RingBuffer case encourages passing owners to observer APIs instead of explicit borrows under the current memory-safety model.

## 修正方針

Rewrite the RingBuffer pipe doctest to borrow for observer calls, e.g. len<i32> &rb and peek<i32> &rb2, then keep ownership release/free behavior correct. Add focused verification for pipe_collections so owner-to-borrow observer regressions stay visible.

## 検証

- `node nodesrc/test_stdlib_ringbuffer_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-ringbuffer-borrow-fix.json -j 1 --dist web/dist`: total=8, passed=8

## 対応結果

`tests/stdlib/pipe_collections.n.md::pipe_ringbuffer_usage` の observer 呼び出しを `len<i32> &rb` と `peek<i32> &rb2` に変更し、観測後に `free<i32> rb` / `free<i32> rb2` で owner を解放するようにした。

`nodesrc/test_stdlib_ringbuffer_borrowed_observers.js` は `tests/stdlib/pipe_collections.n.md` も検査対象に含め、`rb |> peek` のような owner-to-observer pipe が戻らないことを固定した。
