---
id: ISS-20260426T232840301Z-QUEUE-AND-DEQUE-RETAIN-UNSAFE-UNWRAP-50465802
title: "Queue and Deque retain unsafe unwraps in circular-buffer internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md"
---

# ISS-20260426T232840301Z-QUEUE-AND-DEQUE-RETAIN-UNSAFE-UNWRAP-50465802: Queue and Deque retain unsafe unwraps in circular-buffer internals

## 概要

Queue and Deque public APIs return Result for allocation-bearing operations, but circular-buffer internals still call uwok on dealloc_ptr and checked header store_i32 paths in with_capacity, grow, pop, clear, and free. Valid internal state currently relies on unsafe unwraps instead of explicit owner-invariant raw operations.

## 対象

- `stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md`

## 根拠

- `Queue.with_capacity` / `Deque.with_capacity` は data allocation failure 時の header cleanup に `uwok dealloc_ptr` を使っていた。
- `push` / `push_front` / `push_back` grow 経路は old buffer cleanup と header 更新に `uwok` を使っていた。
- `pop` / `clear` / `free` も owned header field 更新や owned buffer cleanup を checked API + unsafe helper で処理していた。

## 問題

Queue and Deque public APIs return Result for allocation-bearing operations, but circular-buffer internals still call uwok on dealloc_ptr and checked header store_i32 paths in with_capacity, grow, pop, clear, and free. Valid internal state currently relies on unsafe unwraps instead of explicit owner-invariant raw operations.

## 影響

Self-host lexer/parser work queues and deques may use these collections as core mutable buffers. Keeping unsafe helpers in normal collection internals violates RV-STDLIB-010 and can turn allocator/header invariant regressions into unreachable traps instead of diagnosable failures.

## 修正方針

Replace Queue/Deque owned-header writes with explicit raw header store helpers, replace owned buffer cleanup with dealloc_raw, keep allocation failures as Result errors, and add source/test regressions that prevent unsafe unwrap helpers from returning to implementation code.

## 解決内容

- `queue_store_header_i32` / `deque_store_header_i32` を追加し、Queue/Deque が所有する header field への内部書き込みを raw store に集約した。
- `with_capacity` の data allocation failure cleanup、grow 後の old buffer cleanup、`free` の owned storage cleanup を `dealloc_raw` に置き換えた。
- `push` / `pop` / `clear` 系の header 更新から `uwok store_i32` を削除した。
- capacity 1 から grow する Queue/Deque の focused regression を追加し、grow 後の順序、clear、free を確認した。
- `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` を追加し、CI source policy と `doc/testing.md` に登録した。

## 検証

- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/queue_collections.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/queue-deque-unsafe-focused-2.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/queue.nepl -i stdlib/alloc/collections/deque.nepl --no-tree -o tmp/queue-deque-unsafe-docs-2.json -j 1`: 16/16 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-queue-deque-unsafe.json -j 4`: 286/286 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-queue-deque-unsafe.json -j 4`: 416/416 passed
- `node nodesrc/issues.js check`: pass
