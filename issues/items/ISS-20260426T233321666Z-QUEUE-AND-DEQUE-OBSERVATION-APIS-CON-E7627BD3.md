---
id: ISS-20260426T233321666Z-QUEUE-AND-DEQUE-OBSERVATION-APIS-CON-E7627BD3
title: "Queue and Deque observation APIs consume owner handles"
area: stdlib
status: open
resolved: false
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

## 検証

Add compile/runtime doctests that call len/peek/is_empty through borrowed observation APIs and then clear/free the same Queue/Deque owner. Keep move-check tests ensuring mutating operations still cannot be used after owner-consuming calls without rebinding.
