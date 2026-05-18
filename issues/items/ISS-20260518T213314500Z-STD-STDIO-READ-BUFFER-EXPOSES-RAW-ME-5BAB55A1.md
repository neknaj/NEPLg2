---
id: ISS-20260518T213314500Z-STD-STDIO-READ-BUFFER-EXPOSES-RAW-ME-5BAB55A1
title: "std/stdio read buffer exposes raw MemPtr fd_read helper"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/std/stdio/read/buffer.nepl, stdlib/std/stdio/read/bytes.nepl, stdlib/std/stdio/read/text.nepl, nodesrc/test_stdlib_stdio_read_boundary.js, tests/stdlib/stdio_read_raw_boundary.n.md"
---

# ISS-20260518T213314500Z-STD-STDIO-READ-BUFFER-EXPOSES-RAW-ME-5BAB55A1: std/stdio read buffer exposes raw MemPtr fd_read helper

## 概要

std/stdio/read/buffer exposes a public fd_read helper that accepts arbitrary MemPtr<u8> iov/nread/data pointers plus a caller-chosen byte length. Direct import callers can bypass the RegionToken-backed read buffer boundary.

## 対象

- `stdlib/std/stdio/read/buffer.nepl, stdlib/std/stdio/read/bytes.nepl, stdlib/std/stdio/read/text.nepl, nodesrc/test_stdlib_stdio_read_boundary.js, tests/stdlib/stdio_read_raw_boundary.n.md`

## 根拠

- `std/stdio/read/buffer.nepl` は direct import 可能な module であり、`pub fn stdio_fd_read_into_result(i32, MemPtr<u8>, MemPtr<u8>, MemPtr<u8>, i32)` を公開していた。
- `std/stdio/read/bytes.nepl` と `std/stdio/read/text.nepl` は自前で `region_ptr` / `mem_ptr_add` により fd_read destination pointer を作り、public raw helper に渡していた。
- Stage 6 の `MemPtr = non-owning pointer` 方針では、fd_read の writable extent は caller convention や lower-level public slice wrapper ではなく、source object と所有権境界内の local owner proof から導出する必要がある。

## 問題

std/stdio/read/buffer exposes a public fd_read helper that accepts arbitrary MemPtr<u8> iov/nread/data pointers plus a caller-chosen byte length. Direct import callers can bypass the RegionToken-backed read buffer boundary.

## 影響

The fd_read readable/writeable extent proof remains a caller convention instead of being derived from owned RegionToken buffers, weakening Stage 6 MemPtr = non-owning pointer discipline.

## 修正方針

Keep the raw MemPtr fd_read helper private, expose high-level ByteBuf read boundaries, and migrate read_all/read_line so iov, nread, and data pointers are derived only inside stdio/read/buffer.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

Run stdio read source policy plus focused stdio read doctests and compile-fail regressions proving the raw MemPtr helper is not importable.

## 対応

- `stdio_fd_read_into_result` を `stdlib/std/stdio/read/buffer.nepl` 内の private helper に戻した。
- いったん検討した public `RegionToken` slice wrapper は、borrowed token 越しの external IO payload extent proof を API 利用者へ押し出すため破棄した。
- public surface は `stdio_read_all_buffer_result` / `stdio_read_line_buffer_result` に限定し、iov / nread scratch、growable/line buffer、`MemPtr` view 導出、fd_read loop、cleanup、`ByteBuf` finalization を `read/buffer` の local owner 境界へ集約した。
- `stdlib/std/stdio/read/bytes.nepl` の `stdio_read_all_bytes_result` は `stdio_read_all_buffer_result` へ委譲し、buffer module 外で `MemPtr` destination span を作らない。
- `stdlib/std/stdio/read/text.nepl` の `stdio_read_line_result` は `stdio_read_line_buffer_result` の `ByteBuf` を UTF-8 検証へ渡すだけにし、raw fd_read helper、scratch allocation、byte inspection を持たない。
- `tests/stdlib/stdio_read_raw_boundary.n.md` を追加し、`std/stdio/read` と direct `std/stdio/read/buffer` import の両方で raw helper が未定義になること、lower-level fd_read slice wrapper も公開されないことを compile-fail regression にした。
- `nodesrc/test_stdlib_stdio_read_boundary.js` を更新し、raw helper / lower-level wrapper の public 化、read_all/read_line 側での raw fd_read span 再構築を拒否する。

## 検証結果

- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/test_stdlib_documentation_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl --no-tree -o tmp/agent1-stdio-read-buffer-doc.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/stdio_read_raw_boundary.n.md --no-tree -o tmp/agent1-stdio-read-raw-boundary.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/std/stdio/read.nepl -i tests/stdlib/stdio_result_stderr.n.md --no-tree -o tmp/agent1-stdio-read-typed-boundary.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
