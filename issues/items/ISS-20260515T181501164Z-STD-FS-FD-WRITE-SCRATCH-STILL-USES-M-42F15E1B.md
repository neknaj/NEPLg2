---
id: ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B
title: "std fs fd_write scratch still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/fs/write/fd.nepl; stdlib/std/fs/raw/fd_io.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B: std fs fd_write scratch still uses MemPtr owner API

## 概要

std/fs/write/fd.nepl still allocates fd_write iovec and nwritten scratch with alloc_ptr/dealloc_ptr, leaving temporary free obligation in MemPtr even though Stage 6 treats MemPtr as non-owning.

## 対象

- `stdlib/std/fs/write/fd.nepl; stdlib/std/fs/raw/fd_io.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/fs/write/fd.nepl` の `fs_write_fd_mem_result` は iovec 8 byte と `nwritten` 4 byte の scratch を `alloc_ptr<u8>` で確保し、終了時に `dealloc_ptr<u8>` へ渡していた。
- Stage 6 の設計では `MemPtr<T>` は non-owning pointer view であり、scratch storage の free obligation owner は `RegionToken` / storage token 側で持つ必要がある。
- `std/stdio/write/fd.nepl` は同じ fd_write scratch を既に `RegionToken<u8>` owner 境界へ移しており、`std/fs/write/fd.nepl` だけが stale `MemPtr` owner API を残していた。

## 問題

std/fs/write/fd.nepl still allocates fd_write iovec and nwritten scratch with alloc_ptr/dealloc_ptr, leaving temporary free obligation in MemPtr even though Stage 6 treats MemPtr as non-owning.

## 影響

std/fs write path keeps Resource IR owner-summary special cases for MemPtr scratch owners and remains inconsistent with the RegionToken fd_write boundary already used by stdio.

## 修正方針

Move fd_write scratch allocation in fs_write_fd_mem_result to RegionToken<u8>, keep raw ABI layout in std/fs/raw/fd_io.nepl, and update source policy to reject direct alloc_ptr/dealloc_ptr in std/fs/write/fd.nepl.

## 検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/std/fs/write/fd.nepl -i stdlib/std/fs/write/path.nepl -i stdlib/std/fs/write.nepl --no-tree -o tmp/agent1-fs-write-fd-region-scratch.json -j 1 --dist web/dist --assert-io`: 2 passed

## 関連

- 親 issue: [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md)
