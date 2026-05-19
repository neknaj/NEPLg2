---
id: ISS-20260519T195128604Z-STD-STDIO-RAW-EXPOSES-PUBLIC-MEMPTR--732B30C2
title: "std/stdio/raw exposes public MemPtr fd I/O helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/std/stdio/raw.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/stdio/read/buffer.nepl"
---

# ISS-20260519T195128604Z-STD-STDIO-RAW-EXPOSES-PUBLIC-MEMPTR--732B30C2: std/stdio/raw exposes public MemPtr fd I/O helpers

## 概要

std/stdio/raw still publishes MemPtr-based fd_read/fd_write helpers and stdio_fd_write_from_result. Ordinary source can direct-import std/stdio/raw and pass caller-selected MemPtr/length pairs, bypassing the typed str/ByteBuf/ByteBuilder public boundaries.

## 対象

- `stdlib/std/stdio/raw.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/stdio/read/buffer.nepl`

## 根拠

- `stdlib/std/stdio/raw.nepl` に `pub fn stdio_fd_read_mem` / `pub fn stdio_fd_write_mem` / `pub fn stdio_fd_write_from_result` が残っていた。
- `std/stdio` root facade は raw module を再公開しないが、ordinary source は `#import "std/stdio/raw" as raw` で explicit direct import できた。
- `stdio_fd_write_from_result` の doctest 自体が、ordinary source から `alloc_region` で得た `MemPtr` と長さを組み合わせて raw fd write helper を呼ぶ成功例になっていた。

## 問題

std/stdio/raw still publishes MemPtr-based fd_read/fd_write helpers and stdio_fd_write_from_result. Ordinary source can direct-import std/stdio/raw and pass caller-selected MemPtr/length pairs, bypassing the typed str/ByteBuf/ByteBuilder public boundaries.

## 影響

Stage 6 MemPtr=non-owning and RegionToken=owner separation remains dependent on caller discipline for stdio fd I/O. This can hide extent/provenance mistakes behind a raw ABI module import instead of forcing source-object-derived proof.

## 修正方針

Move MemPtr fd I/O layout helpers into the owner-boundary modules that allocate the scratch RegionToken values, keep only raw ABI extern/fallback functions in std/stdio/raw, and add source policy plus compile_fail regressions for direct import.

## 検証

Run stdio source policy and focused doctests that direct-import std/stdio/raw, std/stdio/write/fd, and std/stdio/read/buffer to ensure raw span helpers are not callable.

## 解決内容

- `std/stdio/raw` から `MemPtr` を受ける fd read/write wrapper と fd_write layout helper を削除した。
- raw module に残す公開 API は `stdio_fd_read_raw(fd, iov_raw, iovcnt, nread_raw)` / `stdio_fd_write_raw(fd, iov_raw, iovcnt, nwritten_raw)` の raw `i32` ABI wrapper だけにした。`MemPtr` から raw address への変換と iovec/nread/nwritten layout は owner-boundary module 側に集約する。
- `stdio_fd_read_into_result` は `std/stdio/read/buffer.nepl` 内の private helper として local `RegionToken<u8>` scratch から raw address を導出し、`stdio_fd_read_raw` だけを呼ぶ。
- `stdio_fd_write_from_result` は `std/stdio/write/fd.nepl` 内の private helper へ移し、typed public wrapper (`str` / `ByteBuf` / `ByteBuilder` / 1 byte) から導出された readable span だけを fd write loop へ渡す。
- `nodesrc/test_stdlib_stdio_read_boundary.js` は、`std/stdio/raw` に `MemPtr` wrapper が戻らないこと、raw `i32` ABI wrapper だけが残ること、write/read 側 private helper が public 化されないことを検査する。
- `tests/stdlib/stdio_raw_boundary.n.md` を追加し、direct import で旧 raw helper と private layout helper が未定義になることを compile_fail で固定した。

## focused verification

- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdio_raw_boundary.n.md --no-tree -o tmp/agent1-stdio-raw-boundary.json -j 1 --dist web/dist --assert-io`: 5/5 passed
- `node nodesrc/tests.js -i stdlib/std/stdio/write/fd.nepl -i stdlib/std/stdio/read/buffer.nepl --no-tree -o tmp/agent1-stdio-boundary-fd-buffer.json -j 1 --dist web/dist --assert-io`: 3/3 passed

## 関連

- parent: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- related fs boundary: `ISS-20260519T193523555Z-STD-FS-FD-IO-RAW-MEMPTR-HELPERS-REMA-11E47D1F`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
