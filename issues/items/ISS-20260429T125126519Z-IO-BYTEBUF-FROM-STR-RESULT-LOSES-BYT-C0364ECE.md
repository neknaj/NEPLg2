---
id: ISS-20260429T125126519Z-IO-BYTEBUF-FROM-STR-RESULT-LOSES-BYT-C0364ECE
title: "io_bytebuf_from_str_result loses ByteBuf owner under Resource IR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/io.nepl, tests/stdlib/streamio.n.md"
---

# ISS-20260429T125126519Z-IO-BYTEBUF-FROM-STR-RESULT-LOSES-BYT-C0364ECE: io_bytebuf_from_str_result loses ByteBuf owner under Resource IR

## 概要

After origin/main 78f310e, tests/stdlib/streamio.n.md reports io_bytebuf_from_str_result ConstructInput on ByteBuf out found Moved and out_raw owner may leak. The conversion allocates/copies bytes but the owner transfer into ByteBuf is not ResourceIR-clean.

## 対象

- `stdlib/alloc/io.nepl, tests/stdlib/streamio.n.md`

## 根拠

- `tests/stdlib/streamio.n.md` の text/binary stream roundtrip が `io_bytebuf_from_str_result` の `Result::Ok(ByteBuf).ptr` owner を戻り値へ移せず、`ConstructInput` / `ReturnValue` の owner violation で止まっていた。
- `io_bytebuf_from_str_result` は `alloc_ptr<u8>` で得た `MemPtr` を `out_raw` / `data_raw` の raw address に分解してから `ByteBuf out byte_len` へ戻しており、Resource IR から見ると owner が raw address local と aggregate field の間で分断されていた。
- 空 `ByteBuf` も `MemPtr(0)` と `len=0` の同じ `ptr` field で表していたため、所有領域がある buffer とない buffer の区別が型に現れていなかった。
- `fs_read_fd_bytes` は 0 byte 読み取り成功時にも scratch buffer owner を確保したまま `ByteBuf len=0` へ丸めており、空 buffer と所有領域の区別がない設計に依存していた。

## 問題

After origin/main 78f310e, tests/stdlib/streamio.n.md reports io_bytebuf_from_str_result ConstructInput on ByteBuf out found Moved and out_raw owner may leak. The conversion allocates/copies bytes but the owner transfer into ByteBuf is not ResourceIR-clean.

## 影響

String-to-byte output helpers used by streamio and self-host IO cannot pass strict memory-safety checking. This blocks buffered output tests independently of StreamWriter header raw loads.

## 修正方針

Review alloc/io string-to-ByteBuf construction. Preserve the output allocation owner until the ByteBuf value is constructed exactly once, and ensure all allocation/copy failure paths free the output region or avoid creating an owner obligation.

## 修正内容

- `ByteBuf.ptr` を `Option<MemPtr<u8>>` に変更し、空 buffer は `None`、所有領域を持つ buffer は `Some(ptr)` として表すようにした。null pointer を所有 field として混ぜる設計をやめた。
- `io_bytebuf_alloc_region` / `io_bytebuf_region_ptr` / `io_bytebuf_finish_region` を追加し、`io_bytebuf_from_str_result` は `RegionToken<u8>` で確保領域 owner を保持したまま copy し、最後に `ByteBuf` へ一度だけ移す形へ変更した。
- `io_bytebuf_from_owned_ptr`、`io_bytebuf_len_ref`、`io_bytebuf_ptr_ref`、`io_bytebuf_byte_at` を追加し、`std/stdio` / `std/fs` / `std/text` / `std/streamio` の direct field read と direct `ByteBuf ptr len` construction を置き換えた。
- `fs_read_fd_bytes` の 0 byte 成功読み取りは scratch buffer を解放してから `io_bytebuf_empty` を返すようにし、空 `ByteBuf` が隠れた owner を持たないことを保証した。
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` を追加し、`io_bytebuf_from_str_result` が raw pointer intermediate や直接 `alloc_ptr` に戻らないこと、`ByteBuf` が `Option<MemPtr<u8>>` で空/所有領域を区別することを固定した。
- 既存 `.n.md` の direct `ByteBuf data len` / `get bytes "ptr"` は public helper 経由に更新した。

## 検証

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-bytebuf-owner-after.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/bytebuf-result-owner-after.json -j 1 --dist web/dist`: total=6, passed=5, failed=1。`io_bytebuf_from_str_result` failure は解消済み。残件は既存 `ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D` の `fs_open_with_flags` out pointer ResourceIR failure。
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-after-alloc-io-bytebuf.json -j 1 --dist web/dist`: total=14, passed=8, failed=6。doctest #2/#3 の `io_bytebuf_from_str_result` failure は解消済み。残件は既存 `stdio_finish_read_buffer` / `std fs WASI out pointer` issues。
