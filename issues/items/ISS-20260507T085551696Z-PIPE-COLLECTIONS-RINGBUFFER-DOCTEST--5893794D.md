---
id: ISS-20260507T085551696Z-PIPE-COLLECTIONS-RINGBUFFER-DOCTEST--5893794D
title: "pipe_collections RingBuffer doctest passes owner to borrow-only APIs"
area: TEST
status: open
resolved: false
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

- 未記入

## 問題

tests/stdlib/pipe_collections.n.md::doctest#7 still calls len<i32> rb and rb2 |> peek after RingBuffer len/peek are borrow-only APIs. The selected std/test import failure is fixed, but this stale fixture now fails with type.overload.no_match and hides RingBuffer pipe coverage.

## 影響

The pipe_collections suite cannot fully validate collection pipe usage, and the failing RingBuffer case encourages passing owners to observer APIs instead of explicit borrows under the current memory-safety model.

## 修正方針

Rewrite the RingBuffer pipe doctest to borrow for observer calls, e.g. len<i32> &rb and peek<i32> &rb2, then keep ownership release/free behavior correct. Add focused verification for pipe_collections so owner-to-borrow observer regressions stay visible.

## 検証

node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-ringbuffer-borrow-fix.json -j 1 --dist web/dist
