---
id: ISS-20260515T201227745Z-STD-FS-DIR-READ-SCRATCH-STILL-USES-M-92BCD4BA
title: "std/fs dir read scratch still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/fs/dir/read_fd.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T201227745Z-STD-FS-DIR-READ-SCRATCH-STILL-USES-M-92BCD4BA: std/fs dir read scratch still uses MemPtr owner API

## 概要

std/fs/dir/read_fd.nepl still allocates fd_readdir buffer and bufused scratch through alloc_ptr/dealloc_ptr, so directory reading keeps free obligation in MemPtr even though Stage 6 treats MemPtr as a non-owning view.

## 対象

- `stdlib/std/fs/dir/read_fd.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/fs/dir/read_fd.nepl` は `fd_readdir` の data buffer と `used` out-pointer scratch を `alloc_ptr<u8>` / `dealloc_ptr<u8>` で確保していた。
- Stage 6 の方針では `MemPtr<T>` は non-owning pointer view であり、free obligation owner は `RegionToken` / storage owner 側へ分離する必要がある。
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` には directory fd reader がこの低レベル owner API を再導入しない policy がまだ無かった。

## 問題

std/fs/dir/read_fd.nepl still allocates fd_readdir buffer and bufused scratch through alloc_ptr/dealloc_ptr, so directory reading keeps free obligation in MemPtr even though Stage 6 treats MemPtr as a non-owning view.

## 影響

The directory listing path remains an exception to the RegionToken owner boundary used by fs read/write/open/stat and keeps the public alloc_ptr migration parent open with stale MemPtr owner cleanup.

## 修正方針

Move fd_readdir data and bufused scratch to RegionToken<u8> owners, derive only non-owning MemPtr/raw views for the ABI parse loop, and update source policy so the direct MemPtr owner API cannot return.

## 検証

Run fs source policy, directory facade/consumer doctest, issues check, and diff whitespace check. `std/fs/dir/read_fd.nepl` itself currently has no runnable doctest, so the direct fd helper is guarded by source policy and the public consumer doctest.

## 解決

2026-05-16 Agent 1 で解決。

- `fs_read_dir_fd` の `fd_readdir` data buffer と `used` out-pointer scratch を `RegionToken<u8>` owner に移した。
- raw ABI へ渡す address は `region_ptr &buf_region` / `region_ptr &used_region` から得た non-owning `MemPtr<u8>` view を `mem_ptr_addr` した値に限定した。
- cleanup は `dealloc_region<u8>` で owner token を消費する形に揃えた。
- source policy に、`std/fs/dir/read_fd.nepl` が `core/mem/pointer/alloc` や `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を再導入しない検査を追加した。
