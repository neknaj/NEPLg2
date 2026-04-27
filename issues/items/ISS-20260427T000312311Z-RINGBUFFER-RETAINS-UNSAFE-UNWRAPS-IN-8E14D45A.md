---
id: ISS-20260427T000312311Z-RINGBUFFER-RETAINS-UNSAFE-UNWRAPS-IN-8E14D45A
title: "RingBuffer retains unsafe unwraps in circular-buffer internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/ringbuffer.nepl, tests/stdlib/ringbuffer_collections.n.md, nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js"
---

# ISS-20260427T000312311Z-RINGBUFFER-RETAINS-UNSAFE-UNWRAPS-IN-8E14D45A: RingBuffer retains unsafe unwraps in circular-buffer internals

## 概要

RingBuffer still uses uwok for header stores, grow cleanup, and free despite owning the header and data buffer.

## 対象

- `stdlib/alloc/collections/ringbuffer.nepl, tests/stdlib/ringbuffer_collections.n.md`

## 根拠

- `RingBuffer` は `with_capacity` で 16 byte の header と `cap * size_of<.T>` byte の element buffer を確保し、その両方を owner handle が単独所有する設計になっている。
- しかし header の `len/cap/head/data` 更新、grow 時の旧 buffer cleanup、`free` の data/header cleanup が `uwok` 経由で checked API の `Result` を unwrap していた。
- owner invariant の内側では pointer と size が実装自身によって構成されるため、失敗時に public helper trap へ落とすより、raw owner access / cleanup として明示する方が設計に合う。

## 問題

RingBuffer still uses uwok for header stores, grow cleanup, and free despite owning the header and data buffer.

## 影響

The lower-level circular buffer used by queues can still regress to unsafe helper traps even after Queue/Deque were fixed.

## 修正方針

Move header writes to raw owner-invariant helpers, use dealloc_raw for owned buffers, add grow/pop/clear/free regressions, and add a source guard.

## 解決内容

- `ringbuffer_store_header_i32` を追加し、header field の owner-internal write を一箇所に集約した。
- `with_capacity` の data allocation failure cleanup、grow 時の旧 data cleanup、`free` の data/header cleanup を `dealloc_raw` に変更した。
- grow 後の `clear` / `free` と、single-element buffer の `free` が trap しないことを確認する regression を追加した。
- RingBuffer 実装に `unwrap` / `unwrap_ok` / `unwrap_err` / `uwok` / `uwerr` / `unreachable` / `dealloc_ptr` が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer.nepl --no-tree -o tmp/ringbuffer-owned-cleanup-docs.json -j 1`: 10/10 passed
- `node nodesrc/tests.js -i tests/stdlib/ringbuffer_collections.n.md -i stdlib/tests/ringbuffer.n.md --no-tree -o tmp/ringbuffer-owned-cleanup-focused-2.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-ringbuffer-owned-cleanup.json -j 4`: 292/292 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-ringbuffer-owned-cleanup.json -j 4`: 418/418 passed
