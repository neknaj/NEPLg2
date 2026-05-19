---
id: ISS-20260519T193523555Z-STD-FS-FD-IO-RAW-MEMPTR-HELPERS-REMA-11E47D1F
title: "std/fs fd_io raw MemPtr helpers remain directly importable"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/std/fs/raw.nepl, stdlib/std/fs/raw/fd_io.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js, tests/stdlib/fs_fd_raw_boundary.n.md"
---

# ISS-20260519T193523555Z-STD-FS-FD-IO-RAW-MEMPTR-HELPERS-REMA-11E47D1F: std/fs fd_io raw MemPtr helpers remain directly importable

## 概要

std/fs/raw/fd_io exposes fs_fd_read_into_result and fs_fd_write_from_result as public helpers through std/fs/raw, so ordinary source can direct-import raw fd_read/fd_write pointer/length boundaries instead of going through ByteBuf/RegionToken-owned read and write APIs.

## 対象

- `stdlib/std/fs/raw.nepl, stdlib/std/fs/raw/fd_io.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js, tests/stdlib/fs_fd_raw_boundary.n.md`

## 根拠

- `stdlib/std/fs/raw/fd_io.nepl` は direct import 可能な module であり、`pub fn fs_fd_read_into_result(i32, MemPtr<u8>, MemPtr<u8>, MemPtr<u8>, i32)` と `pub fn fs_fd_write_from_result(i32, MemPtr<u8>, MemPtr<u8>, MemPtr<u8>, i32)` を公開していた。
- `std/fs/raw.nepl` は `pub #import "std/fs/raw/fd_io" as *` を持っていたため、root `std/fs` facade から raw module を外していても、通常 source が `#import "std/fs/raw" as *` または `#import "std/fs/raw/fd_io" as *` で fd I/O raw span helper へ到達できた。
- Stage 6 の方針では、fd read/write の writable/readable extent は caller-selected `MemPtr` / length pair ではなく、`std/fs/read/fd` / `std/fs/write/fd` が所有する `RegionToken<u8>` scratch と `ByteBuf` owner boundary から導出する必要がある。

## 問題

std/fs/raw/fd_io exposes fs_fd_read_into_result and fs_fd_write_from_result as public helpers through std/fs/raw, so ordinary source can direct-import raw fd_read/fd_write pointer/length boundaries instead of going through ByteBuf/RegionToken-owned read and write APIs.

## 影響

The std/fs fd I/O span proof remains partly an API convention: callers can choose arbitrary MemPtr/iovec/out-pointer combinations, weakening the Stage 6 MemPtr = non-owning view discipline and making later static-check simplification harder.

## 修正方針

Move fd_read/fd_write raw layout helpers into the read/fd and write/fd owner-boundary modules as private functions, remove raw/fd_io from the public raw facade, and keep public fs APIs limited to ByteBuf/RegionToken-backed boundaries.

## 検証

Run fs source policy, focused fs raw boundary compile_fail doctests, focused std/fs read/write doctests, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 対応

- `stdlib/std/fs/raw/fd_io.nepl` を削除し、`std/fs/raw` facade から `raw/fd_io` re-export を外した。
- fd read の `fs_fd_read_into_result`、`fs_discard_read_buffer`、`fs_finish_read_buffer` は `stdlib/std/fs/read/fd.nepl` の private helper に移した。read loop の iovec / nread / growable buffer owner は引き続き local `RegionToken<u8>` が保持し、raw ABI に渡す値は `region_ptr` / `mem_ptr_add` 由来の non-owning view だけにした。
- fd write の `fs_fd_write_from_result` は `stdlib/std/fs/write/fd.nepl` の private helper に移した。write loop の iovec / nwritten owner は local `RegionToken<u8>` が保持し、`ByteBuf` から同時に導出した data pointer / length だけを private raw ABI helper に渡す。
- `tests/stdlib/fs_fd_raw_boundary.n.md` を追加し、`std/fs/raw`、direct `std/fs/read/fd`、direct `std/fs/write/fd` import のいずれからも fd I/O raw helper が見えないことを compile_fail で固定した。
- source policy を更新し、`std/fs/raw/fd_io` file の復活、raw facade re-export、`pub fn fs_fd_read_into_result` / `pub fn fs_fd_write_from_result` の再導入を拒否する。

## 検証結果

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/fs_fd_raw_boundary.n.md --no-tree -o tmp/agent1-fs-fd-raw-boundary.json -j 1 --dist web/dist --assert-io`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/std/fs/read/fd.nepl -i stdlib/std/fs/write/fd.nepl -i tests/stdlib/fs_write_raw_boundary.n.md --no-tree -o tmp/agent1-fs-fd-raw-boundary-read-write.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
