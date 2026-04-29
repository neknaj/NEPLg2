---
id: ISS-20260429T124935636Z-STDIO-READ-BUFFER-FINISH-LEAKS-BYTEB-84BA0440
title: "stdio read buffer finish leaks ByteBuf owner under Resource IR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/stdio.nepl, tests/stdlib/stdin.n.md"
---

# ISS-20260429T124935636Z-STDIO-READ-BUFFER-FINISH-LEAKS-BYTEB-84BA0440: stdio read buffer finish leaks ByteBuf owner under Resource IR

## 概要

After origin/main 78f310e, tests/stdlib/stdin.n.md fails 5/5 before runtime. Resource IR reports stdio_finish_read_buffer BranchValue on Result::Ok ByteBuf payload found MaybeFreed, and callers leak buf/iov/nread owner obligations.

## 対象

- `stdlib/std/stdio.nepl, tests/stdlib/stdin.n.md`

## 根拠

- 未記入

## 問題

After origin/main 78f310e, tests/stdlib/stdin.n.md fails 5/5 before runtime. Resource IR reports stdio_finish_read_buffer BranchValue on Result::Ok ByteBuf payload found MaybeFreed, and callers leak buf/iov/nread owner obligations.

## 影響

stdin read fixtures and streamio tests are no longer clean gates after the stricter owner checker. The stdio read boundary cannot be trusted for self-host input until the ByteBuf owner transfer and cleanup paths are explicit.

## 修正方針

Review stdio_finish_read_buffer and its callers. Keep the exact-size ByteBuf invariant, but redesign the API so each path either transfers the buffer owner into the returned ByteBuf or frees every allocated scratch region without merging MaybeFreed owner states into the success value.

## 検証

Run node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-owner-after.json -j 1 --dist web/dist and node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/stdin-after-stdio-owner.json -j 1 --dist web/dist.
