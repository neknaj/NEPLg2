---
id: ISS-20260518T210549005Z-STD-STDIO-WRITE-FACADE-EXPOSES-RAW-M-11591E6E
title: "std/stdio write facade exposes raw MemPtr span writers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/std/stdio/write/fd.nepl, stdlib/std/stdio/write/text.nepl, stdlib/std/stdio/write/bytes.nepl, stdlib/std/stdio/write/byte.nepl, stdlib/std/stdio/print.nepl, stdlib/std/streamio/writer/state.nepl, nodesrc/test_stdlib_stdio_read_boundary.js, nodesrc/test_stdlib_streamio_writer_boundary.js, tests/stdlib/stdio_write_raw_boundary.n.md"
---

# ISS-20260518T210549005Z-STD-STDIO-WRITE-FACADE-EXPOSES-RAW-M-11591E6E: std/stdio write facade exposes raw MemPtr span writers

## 概要

std/stdio/write re-exports write/fd while write/fd exposes public functions that accept MemPtr<u8> plus an arbitrary byte length. Ordinary source can bypass the str, ByteBuf, or ByteBuilder owner boundaries and ask the compiler-owned fd_write ABI helper to read a caller-chosen span.

## 対象

- `stdlib/std/stdio/write/fd.nepl, stdlib/std/stdio/write/text.nepl, stdlib/std/stdio/write/bytes.nepl, stdlib/std/stdio/write/byte.nepl, stdlib/std/stdio/print.nepl, stdlib/std/streamio/writer/state.nepl, nodesrc/test_stdlib_stdio_read_boundary.js, nodesrc/test_stdlib_streamio_writer_boundary.js, tests/stdlib/stdio_write_raw_boundary.n.md`

## 根拠

- `std/stdio/write` facade は `std/stdio/write/fd` を `@merge` しており、`pub fn stdio_write_fd_mem_result(i32, MemPtr<u8>, i32)` / `stdio_write_mem_result` / `stdio_write_stderr_mem_result` / unit wrapper が direct import caller から見えていた。
- `std/streamio/writer/state.nepl` は `ByteBuilder` の `byte_builder_data_ptr_ref` と `stdio_write_mem` / `stdio_write_stderr_mem` を組み合わせて flush しており、writer state が raw pointer span を構成していた。
- Stage 6 の `MemPtr = non-owning pointer` 方針では、public API が任意 pointer/length pair を受けると readable extent の証明が caller convention へ落ちる。

## 問題

std/stdio/write re-exports write/fd while write/fd exposes public functions that accept MemPtr<u8> plus an arbitrary byte length. Ordinary source can bypass the str, ByteBuf, or ByteBuilder owner boundaries and ask the compiler-owned fd_write ABI helper to read a caller-chosen span.

## 影響

The stdout/stderr write surface leaves readable extent as a caller convention instead of a typed source proof. This weakens Stage 6's MemPtr = non-owning pointer discipline and can hide unsafe pointer/length pairing behind public stdio helpers.

## 修正方針

Keep the fd_write raw span loop as a private helper, expose typed wrappers for str, ByteBuf, ByteBuilder prefix, and single byte output, and migrate print/text/bytes/streamio callers to those wrappers without stdlib allowlists.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

Run stdio and streamio source policies plus focused stdio/streamio doctests and compile-fail regressions proving public raw MemPtr write helpers are not importable.

## 対応

- `stdio_write_fd_mem_result` を `stdlib/std/stdio/write/fd.nepl` 内の private helper に戻し、public fd write surface を typed wrapper に限定した。
- `str` は `stdio_write_fd_str_result(fd, s)`、`ByteBuf` は `stdio_write_fd_bytebuf_result(fd, bytes)`、borrowed `ByteBuilder` flush は `stdio_write_fd_bytebuilder_prefix_result(fd, &builder, byte_len)`、1 byte output は `stdio_write_fd_byte_result(fd, b)` に分離した。
- `stdio_write_fd_bytebuilder_prefix_result` は `0 <= byte_len <= builder.len` を確認し、正の byte_len で storage view が取れない場合は `InvalidOperation` を返す。
- `std/stdio/write/text` / `bytes` / `byte` / `print` / `std/streamio/writer/state` を typed wrapper 経由に移行し、streamio writer が raw `ByteBuilder` pointer を直接取り出さないようにした。
- `tests/stdlib/stdio_write_raw_boundary.n.md` を追加し、safe facade と direct fd module import の両方で raw span writer が undefined になることを compile-fail regression にした。
- `nodesrc/test_stdlib_stdio_read_boundary.js` と `nodesrc/test_stdlib_streamio_writer_boundary.js` を更新し、raw span public helper の再導入、text/bytes/streamio 側での raw pointer span 再構築を拒否する。

## 検証結果

- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdio_write_raw_boundary.n.md --no-tree -o tmp/agent1-stdio-write-raw-boundary.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/std/stdio/write.nepl -i tests/stdlib/stdio_result_stderr.n.md --no-tree -o tmp/agent1-stdio-write-typed-boundary.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-streamio-stdio-write-boundary-j4.json -j 4 --dist web/dist --assert-io`: total=15, passed=15
