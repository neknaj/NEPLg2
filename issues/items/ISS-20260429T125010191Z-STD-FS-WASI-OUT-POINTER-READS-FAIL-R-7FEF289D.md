---
id: ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D
title: "std fs WASI out pointer reads fail RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
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

- 未記入

## 問題

After origin/main 78f310e, doctests that import std/fs fail before runtime. Resource IR reports RawMemoryLoadCell Uninit at fs_open_with_flags load_i32 fd_out and fs_read_fd_bytes load_i32 nread.

## 影響

File input helpers block stdin/streamio regression runs and self-host file IO validation under strict ResourceIR checking. Leaving the raw out-pointer pattern would keep WASI output initialization outside the checker-visible boundary.

## 修正方針

Redesign fs WASI call boundaries like the stdio read boundary: initialize each scratch/out pointer and read it back inside a small operation-specific helper, or introduce a typed out-param abstraction that preserves initialization provenance without generic raw load helpers.

## 検証

Run node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/fs-raw-outparam-after.json -j 1 --dist web/dist plus stdin/streamio focused fixtures that previously reported fs_open_with_flags and fs_read_fd_bytes.
