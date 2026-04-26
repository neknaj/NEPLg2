---
id: ISS-20260426T233321666Z-QUEUE-AND-DEQUE-OBSERVATION-APIS-CON-E7627BD3
title: "Queue and Deque observation APIs consume owner handles"
area: stdlib
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-27
target: "stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md"
---

# ISS-20260426T233321666Z-QUEUE-AND-DEQUE-OBSERVATION-APIS-CON-E7627BD3: Queue and Deque observation APIs consume owner handles

## 概要

Queue and Deque read-oriented APIs such as len, is_empty, peek, peek_front, and peek_back take the collection handle by value. Focused regression attempts to peek/pop and then clear/free the same handle still fail with D3053 use of moved value, so doctests keep duplicated setup variables instead of expressing one natural workflow.

## 対象

- `stdlib/alloc/collections/queue.nepl, stdlib/alloc/collections/deque.nepl, tests/stdlib/queue_collections.n.md, tests/stdlib/deque_collections.n.md`

## 根拠

- Queue/Deque grow regression を 1 つの owner workflow として書いたところ、`pop` / `peek` 後に同じ handle を `clear/free` へ渡す箇所で `D3053 use of moved value` になった。
- `len` / `is_empty` / `peek` / `peek_front` / `peek_back` は読み取り専用に見えるが、現 signature は `Queue<.T>` / `Deque<.T>` を by-value で受ける。
- そのため doctest は同じ構造を複数回 `new` / `push` して観測ごとに別 owner を作る必要があり、古い borrow checker 回避に似た不自然なサンプルが残る。

## 問題

Queue and Deque read-oriented APIs such as len, is_empty, peek, peek_front, and peek_back take the collection handle by value. Focused regression attempts to peek/pop and then clear/free the same handle still fail with D3053 use of moved value, so doctests keep duplicated setup variables instead of expressing one natural workflow.

## 影響

Self-host lexer/parser work queues need frequent observation plus later cleanup. Rebuilding equivalent Queue/Deque values in tests and callers hides ownership intent, leaves old borrow-checker workaround style in examples, and makes future refactoring harder.

## 修正方針

Add borrowed observation APIs or change read-only APIs to take &Queue/&Deque where compatible, keep mutating pop/push ownership semantics explicit, and refactor doctests to reuse one owner handle once the borrowed API is available.

## 解決内容

- `Queue` に `len_ref` / `is_empty_ref` / `peek_ref` を追加し、owner handle を移動せずに length / empty / front value を観測できるようにした。
- `Deque` に `len_ref` / `cap_ref` / `is_empty_ref` / `peek_front_ref` / `peek_back_ref` を追加した。
- borrowed peek 系は owner 内の値を複製して返すため `.T: Copy` bound を付け、mutating `pop` / `push` の owner-consuming semantics は維持した。
- `tests/stdlib/queue_collections.n.md` / `tests/stdlib/deque_collections.n.md` の重複 setup を削除し、同じ owner を `len_ref` / `peek_ref` 後に `clear` / `free` する regression に更新した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/queue_collections.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/queue-deque-borrowed-observation-focused.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/queue.nepl -i stdlib/alloc/collections/deque.nepl --no-tree -o tmp/queue-deque-borrowed-observation-docs.json -j 1`: 16/16 passed
- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-queue-deque-borrowed-observation.json -j 4`: 286/286 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-queue-deque-borrowed-observation.json -j 4`: 416/416 passed
