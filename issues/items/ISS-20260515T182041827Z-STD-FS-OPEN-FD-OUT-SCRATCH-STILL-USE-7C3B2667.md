---
id: ISS-20260515T182041827Z-STD-FS-OPEN-FD-OUT-SCRATCH-STILL-USE-7C3B2667
title: "std fs open fd_out scratch still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/fs/fd.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T182041827Z-STD-FS-OPEN-FD-OUT-SCRATCH-STILL-USE-7C3B2667: std fs open fd_out scratch still uses MemPtr owner API

## 概要

std/fs/fd.nepl still allocates the path_open fd_out scratch buffer with alloc_ptr/dealloc_ptr, leaving a temporary free obligation in MemPtr even though fd_out is just a 4-byte RegionToken-owned syscall out pointer.

## 対象

- `stdlib/std/fs/fd.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/fs/fd.nepl` の `fs_open_with_flags` は `path_open` の fd_out 用 4 byte scratch を `alloc_ptr<u8>` で確保し、終了時に `dealloc_ptr<u8>` へ渡していた。
- Stage 6 では `MemPtr<T>` を non-owning pointer view として扱うため、scratch storage の free obligation は `RegionToken` / storage token に持たせる必要がある。
- `fs_open_with_flags` は `std/fs` facade から再公開される fd lifecycle 境界であり、ここに `MemPtr` owner API が残ると safe surface に近い場所へ古い owner model が残る。

## 問題

std/fs/fd.nepl still allocates the path_open fd_out scratch buffer with alloc_ptr/dealloc_ptr, leaving a temporary free obligation in MemPtr even though fd_out is just a 4-byte RegionToken-owned syscall out pointer.

## 影響

The public fs open boundary keeps a MemPtr owner special case in a safe facade re-exported by std/fs, delaying Stage 6 removal of MemPtr as an owner carrier.

## 修正方針

Move fs_open_with_flags fd_out scratch allocation to RegionToken<u8>, keep raw path_open layout in the fd boundary, and update source policy to reject direct alloc_ptr/dealloc_ptr in std/fs/fd.nepl.

## 検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/std/fs/fd.nepl -i stdlib/std/fs/dir/open.nepl --no-tree -o tmp/agent1-fs-open-fdout-region.json -j 1 --dist web/dist --assert-io`: 2 passed

## 関連

- 親 issue: [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md)
