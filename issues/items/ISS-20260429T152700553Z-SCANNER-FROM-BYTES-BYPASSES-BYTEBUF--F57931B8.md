---
id: ISS-20260429T152700553Z-SCANNER-FROM-BYTES-BYPASSES-BYTEBUF--F57931B8
title: "scanner_from_bytes bypasses ByteBuf free boundary on header initialization failures"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/streamio.nepl, nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js"
---

# ISS-20260429T152700553Z-SCANNER-FROM-BYTES-BYPASSES-BYTEBUF--F57931B8: scanner_from_bytes bypasses ByteBuf free boundary on header initialization failures

## 概要

scanner_from_bytes destructured ByteBuf.ptr and cleaned up the extracted buffer pointer directly when scanner header allocation or initialization failed. This bypassed the centralized io_bytebuf_free boundary required by the Option<MemPtr<u8>> ByteBuf model.

## 対象

- `stdlib/std/streamio.nepl, nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`

## 根拠

- CI source policy 相当の `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js` が `scanner_from_bytes must initialize scanner headers through Result-returning stores and clean up on failure` で失敗した。
- `scanner_from_bytes` は `match get bytes "ptr"` で取り出した `buf` を、header allocation / `BufPtr` / `Len` / `Pos` 初期化失敗 path で `dealloc_ptr<u8> buf ...` により直接解放していた。
- `ByteBuf` は [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の方針で `io_bytebuf_free` を owner 消費境界にしているため、StreamScanner 側の failure cleanup も同じ境界を使う必要がある。

## 問題

scanner_from_bytes destructured ByteBuf.ptr and cleaned up the extracted buffer pointer directly when scanner header allocation or initialization failed. This bypassed the centralized io_bytebuf_free boundary required by the Option<MemPtr<u8>> ByteBuf model.

## 影響

StreamScanner construction could drift away from the ByteBuf ownership contract, making header initialization failures and invalid ByteBuf states harder to validate under source policy and Resource IR.

## 修正方針

Keep scanner header initialization through Result-returning stores, but clean up the original ByteBuf with io_bytebuf_free bytes on every failure before returning an error.

## 修正内容

- `scanner_from_bytes` の header allocation failure と header field initialization failure で `io_bytebuf_free bytes` を呼ぶようにした。
- invalid `ByteBuf` state (`ptr=None` かつ `len != 0`) でも、戻る前に `io_bytebuf_free bytes` を通すようにした。
- `Option::Some buf` branch の failure cleanup から raw `dealloc_ptr<u8> buf ...` を削除し、ByteBuf owner contract を `stdlib/alloc/io.nepl` に集約した。

## 検証

- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-scanner-bytebuf-free-boundary.json -j 1 --dist web/dist`: total=14, passed=14
