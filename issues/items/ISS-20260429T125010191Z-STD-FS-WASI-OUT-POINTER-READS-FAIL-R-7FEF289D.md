---
id: ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D
title: "std fs WASI out pointer reads fail RawMemoryLoadCell gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/fs.nepl, tests/stdlib/stdin.n.md, tests/stdlib/streamio.n.md"
---

# ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D: std fs WASI out pointer reads fail RawMemoryLoadCell gate

## 概要

After origin/main 78f310e, doctests that import std/fs fail before runtime. Resource IR reports RawMemoryLoadCell Uninit at fs_open_with_flags load_i32 fd_out and fs_read_fd_bytes load_i32 nread.

## 対象

- `stdlib/std/fs.nepl, tests/stdlib/stdin.n.md, tests/stdlib/streamio.n.md`

## 根拠

- `fs_open_with_flags` は `fd_out` 用 scratch を `MemPtr<i32>` に変換し、`store_i32 fd_out 0` の後に `load_i32 fd_out` で読み戻していた。Resource IR はこの typed pointer 変換越しに同一 scratch cell の初期化を証明できず、`RawMemoryLoadCell ... found Uninit` として拒否していた。
- `fs_read_fd_bytes` も `nread` 用 scratch を `MemPtr<i32>` として関数本体のループで読み戻しており、同じ問題を持っていた。
- さらに `fs_read_fd_bytes` は短い read / EOF の場合でも capacity 65536 の buffer を `ByteBuf len=read_len` として返しており、ByteBuf の exact-size owner invariant が弱かった。

## 問題

After origin/main 78f310e, doctests that import std/fs failed before runtime. Resource IR reported RawMemoryLoadCell Uninit at fs_open_with_flags load_i32 fd_out and fs_read_fd_bytes load_i32 nread.

This is fixed by keeping WASI out pointer scratch initialization and readback inside operation-specific raw-address helpers, without weakening RawMemoryLoadCell.

## 影響

Resolved for fs open/read/write scratch out pointers. `tests/stdlib/stdin.n.md` is clean again. `tests/stdlib/streamio.n.md` still has unrelated owner leaks in assertion/ByteBuf conversion paths.

## 修正方針

Redesigned fs WASI call boundaries like the stdio read boundary:

- `fs_open_with_flags` now initializes and reads `fd_out` through the same raw address local, then frees the scratch buffer.
- `fs_fd_read_into_result` owns the `fd_read` iovec/nread scratch initialization and readback boundary.
- `fs_fd_write_from_result` applies the same boundary to `fd_write`/nwritten, eliminating the same unsafe typed out-pointer pattern from write paths.
- `fs_finish_read_buffer` frees empty reads and shrinks short reads to exact-size `ByteBuf`, so fs read results preserve the ByteBuf owner invariant instead of returning a capacity-sized backing region with a shorter len.

## 検証

- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/fs-raw-outparam-after.json -j 1 --dist web/dist`: `total=7`, `passed=7`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/stdin-after-fs-raw-outparam.json -j 1 --dist web/dist`: `total=5`, `passed=5`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-after-fs-raw-outparam.json -j 1 --dist web/dist`: `total=14`, `passed=11`, `failed=3`; remaining failures are owner leaks in assertion/ByteBuf conversion paths, not fs raw out-pointer loads.
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
