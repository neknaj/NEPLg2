---
id: ISS-20260518T205653919Z-STD-FS-WRITE-FACADE-EXPOSES-RAW-MEMP-A8C64961
title: "std/fs write facade exposes raw MemPtr span writer"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/std/fs/write/fd.nepl, stdlib/std/fs/write.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260518T205653919Z-STD-FS-WRITE-FACADE-EXPOSES-RAW-MEMP-A8C64961: std/fs write facade exposes raw MemPtr span writer

## 概要

std/fs/write/fd.nepl exposes fs_write_fd_mem_result as a public function and std/fs/write re-exports the fd submodule, so ordinary source can direct-import a raw MemPtr<u8> plus arbitrary length write path instead of going through the ByteBuf owner boundary.

## 対象

- `stdlib/std/fs/write/fd.nepl, stdlib/std/fs/write.nepl, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/fs/write.nepl` は `std/fs/write/fd` を再公開している。
- `stdlib/std/fs/write/fd.nepl` の `fs_write_fd_mem_result(i32, MemPtr<u8>, i32)` が public だったため、通常 source が `ByteBuf` owner 境界を通らず raw pointer/length pair を fd write loop に渡せた。

## 問題

std/fs/write/fd.nepl exposes fs_write_fd_mem_result as a public function and std/fs/write re-exports the fd submodule, so ordinary source can direct-import a raw MemPtr<u8> plus arbitrary length write path instead of going through the ByteBuf owner boundary.

## 影響

The filesystem write API lets callers pair a non-owning MemPtr with an unrelated length and ask the compiler-owned raw ABI boundary to read that span. This weakens the Stage 6 MemPtr = non-owning pointer discipline and leaves memory safety dependent on caller convention rather than owner/source proof.

## 修正方針

Make fs_write_fd_mem_result a private fd-module helper, keep public writing through ByteBuf-consuming fs_write_fd_bytes and path/string wrappers, and update source policy plus regression doctests so the raw span writer cannot be re-exported.

## 検証

Run the fs source policy and focused std/fs write doctests, plus a compile-fail regression that direct import of fs_write_fd_mem_result is rejected.

## 対応内容

`fs_write_fd_mem_result` を `std/fs/write/fd.nepl` 内の private helper に戻し、public API は `fs_write_fd_bytes(fd, ByteBuf)` と path/string wrapper に限定した。`fs_write_fd_bytes` は `ByteBuf` から `data` と `data_len` を同時に導出するため、caller が任意の `MemPtr` と任意 length を組み合わせて raw ABI boundary へ渡す経路を閉じる。

`nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` は raw span writer の `pub fn` 再導入を拒否する。`tests/stdlib/fs_write_raw_boundary.n.md` は `std/fs/write` facade と direct `std/fs/write/fd` import の両方で `fs_write_fd_mem_result` が未定義であることを compile-fail として固定する。
