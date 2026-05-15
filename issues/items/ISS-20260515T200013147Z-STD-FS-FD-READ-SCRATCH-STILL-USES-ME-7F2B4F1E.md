---
id: ISS-20260515T200013147Z-STD-FS-FD-READ-SCRATCH-STILL-USES-ME-7F2B4F1E
title: "std/fs fd read scratch still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/fs/read/fd.nepl; stdlib/std/fs/raw/fd_io.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T200013147Z-STD-FS-FD-READ-SCRATCH-STILL-USES-ME-7F2B4F1E: std/fs fd read scratch still uses MemPtr owner API

## 概要

std/fs/read/fd.nepl still allocates the growable read buffer and fd_read iovec/nread scratch through alloc_ptr/realloc_ptr/dealloc_ptr, so the read path models free obligation as MemPtr even though Stage 6 fixes MemPtr as a non-owning view.

## 対象

- `stdlib/std/fs/read/fd.nepl; stdlib/std/fs/raw/fd_io.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は `MemPtr<T>` を non-owning view に固定し、free obligation owner を `RegionToken` / storage owner 側へ分離する。
- `std/fs/read/fd.nepl` は Stage 6 の stdio read / fs write / fs open / fs stat 移行後も、growable read buffer、iovec scratch、nread scratch を `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` で扱っていた。
- `std/fs/raw/fd_io.nepl` の `fs_finish_read_buffer` / `fs_discard_read_buffer` も `MemPtr<u8>` owner を受け取る signature だったため、read path だけ `MemPtr` を free obligation carrier として残していた。

## 問題

std/fs/read/fd.nepl still allocates the growable read buffer and fd_read iovec/nread scratch through alloc_ptr/realloc_ptr/dealloc_ptr, so the read path models free obligation as MemPtr even though Stage 6 fixes MemPtr as a non-owning view.

## 影響

The public fs read path keeps Resource IR owner summaries and stdlib policy dependent on the obsolete MemPtr owner carrier, delaying closure of the public alloc_ptr API issue and leaving read cleanup inconsistent with stdio and fs write/open/stat.

## 修正方針

Move the growable read buffer, iovec, and nread scratch to RegionToken<u8> owners; pass only region_ptr-derived MemPtr views to the raw ABI helper; make fs_finish_read_buffer and fs_discard_read_buffer consume RegionToken owners and use owner-preserving RegionToken realloc for shrink.

## 検証

Run the fs source policy, focused fs read doctest, owner-summary regressions, issues check, and diff whitespace check.

## 2026-05-16 Agent 1 解決メモ

- `fs_read_fd_bytes` の growable buffer、iovec scratch、nread scratch を `alloc_region<u8>` / `RegionToken<u8>` owner に移した。
- raw ABI に渡す値は `region_ptr` と `mem_ptr_add` で得る non-owning `MemPtr<u8>` view に限定し、`MemPtr` を free obligation carrier として保持しない。
- grow 時は `realloc_region_bytes_keep<u8>` を使い、失敗時は `RegionReallocError` から旧 owner を回収して cleanup する。capacity overflow / payload size overflow は allocation failure として扱う。
- `fs_discard_read_buffer` / `fs_finish_read_buffer` は `RegionToken<u8>` owner を消費する signature に変更し、shrink は owner-preserving region realloc、ByteBuf 確定は `io_bytebuf_finish_region` に統一した。
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` に、`std/fs/read/fd` と `std/fs/raw/fd_io` が低レベル `MemPtr` allocation wrapper を使わないこと、RegionToken owner 境界を維持することを追加した。

検証:

- `trunk build`
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `node nodesrc/tests.js -i stdlib/std/fs/read/fd.nepl --no-tree -o tmp/agent1-fs-read-fd-region-token.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/fs/raw/fd_io.nepl --no-tree -o tmp/agent1-fs-raw-fd-io-region-token.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/fs/read.nepl -i stdlib/std/fs/read/path.nepl --no-tree -o tmp/agent1-fs-read-region-token-consumers.json -j 1 --dist web/dist --assert-io`
