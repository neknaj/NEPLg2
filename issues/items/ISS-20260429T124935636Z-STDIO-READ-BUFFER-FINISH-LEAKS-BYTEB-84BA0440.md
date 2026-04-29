---
id: ISS-20260429T124935636Z-STDIO-READ-BUFFER-FINISH-LEAKS-BYTEB-84BA0440
title: "stdio read buffer finish leaks ByteBuf owner under Resource IR"
area: stdlib
status: fixed
resolved: true
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

- 2026-04-29 の Resource IR owner fix で `MaybeFreed` が storage provenance 付きの条件付き owner として caller へ伝播され、値としての move は可能、dealloc/release は不可のまま扱われるようになった。
- これにより `stdio_finish_read_buffer` の success path で `Result::Ok ByteBuf` payload が `BranchValue ... MaybeFreed` として拒否される false positive は解消した。
- 現在の `tests/stdlib/stdin.n.md` に残る失敗は `fs_open_with_flags` の `RawMemoryLoadCell ... found Uninit` であり、本 issue の `stdio_finish_read_buffer` owner leak ではない。

## 問題

After origin/main 78f310e, tests/stdlib/stdin.n.md failed 5/5 before runtime. Resource IR reported stdio_finish_read_buffer BranchValue on Result::Ok ByteBuf payload found MaybeFreed, and callers leaked buf/iov/nread owner obligations.

This is now fixed by the Resource IR owner state work that made `MaybeFreed` a movable conditional owner with summary propagation. The remaining stdin failure is tracked separately by [ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D](./ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D.md).

## 影響

Resolved for the stdio read boundary. Stdin-related full fixtures still depend on fs raw out pointer cleanup and the broader ByteBuf structural invariant issue.

## 修正方針

No stdlib code change is required for this issue after the Resource IR fix. Keep `stdio_finish_read_buffer` as the exact-size ByteBuf boundary: invalid/empty/error paths free the input buffer, success paths transfer the buffer owner into the returned ByteBuf. Broader ByteBuf representation cleanup remains tracked by [ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2](./ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2.md).

## 検証

- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-owner-before.json -j 1 --dist web/dist`: `total=28`, `passed=28`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/stdin-before-stdio-owner.json -j 1 --dist web/dist`: `total=5`, `passed=4`, `failed=1`; remaining failure is `fs_open_with_flags` RawMemoryLoadCell and not `stdio_finish_read_buffer`.
