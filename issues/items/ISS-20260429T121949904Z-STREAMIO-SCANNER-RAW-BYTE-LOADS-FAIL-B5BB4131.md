---
id: ISS-20260429T121949904Z-STREAMIO-SCANNER-RAW-BYTE-LOADS-FAIL-B5BB4131
title: "streamio scanner raw byte loads fail RawMemoryLoadCell gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/streamio.nepl, tests/stdlib/stdin.n.md"
---

# ISS-20260429T121949904Z-STREAMIO-SCANNER-RAW-BYTE-LOADS-FAIL-B5BB4131: streamio scanner raw byte loads fail RawMemoryLoadCell gate

## 概要

tests/stdlib/stdin.n.md doctest#5 fails in streamio scanner code under the RawMemoryLoadCell gate. The diagnostics point to stream_scanner_load_header_result, stream_scanner_skip_ws_header, and scan_i32_impl reading scanner header/buffer raw cells as Uninit.

## 対象

- `stdlib/std/streamio.nepl, tests/stdlib/stdin.n.md`

## 根拠

- `tests/stdlib/stdin.n.md::doctest#5` は `scan_i32_impl`、`stream_scanner_load_header_result`、`stream_scanner_skip_ws_header` で `RawMemoryLoadCell ... found Uninit` を報告していた。
- `stream_scanner_load_header_result` は `region_ptr_at` で作った `MemPtr<i32>` を任意 header pointer から読み、scanner 初期化時の header store との関係を ResourceIR が追跡できなかった。
- scanner buffer は header 内の raw `i32` address から読み出され、その後 `load_u8 add buf p` で直接読まれていたため、`ByteBuf` として初期化済みである provenance が失われていた。

## 問題

tests/stdlib/stdin.n.md doctest#5 fails in streamio scanner code under the RawMemoryLoadCell gate. The diagnostics point to stream_scanner_load_header_result, stream_scanner_skip_ws_header, and scan_i32_impl reading scanner header/buffer raw cells as Uninit.

## 影響

stdin scanner tests are not a clean regression target, and self-host / tutorial code that depends on StreamScanner cannot rely on streamio while strict raw memory initialization checking is enabled.

## 修正方針

Review std/streamio scanner header and byte access boundaries. Do not weaken RawMemoryLoadCell; replace generic raw header/byte loads or move initialization/read operations into boundaries that preserve ResourceIR initialization provenance.

## 修正内容

- scanner header load/store は `region_ptr_at` で `MemPtr<i32>` を作る helper を廃止し、scanner state boundary として raw offset を直接扱う形に整理した。
- scanner buffer は raw `i32` のまま読まず、`MemPtr<u8>` に戻して `stream_scanner_byte_at` へ byte access を集約した。
- `skip_ws` / `skip` / `scan_token_impl` / `scan_i32_impl` / `scan_u32_impl` / `scan_u64_impl` / `scan_i64_impl` / `scan_f64_impl` の raw `load_u8 add buf ...` を削除した。
- `scan_token_impl` は token string layout を手作りせず、`string_from_mem_unchecked_result (mem_ptr_add buf start) tlen` へ委譲した。
- `nodesrc/test_stdlib_streamio_scanner_boundary.js` を追加し、scanner raw byte load と raw string layout の再導入を禁止した。

## 検証

- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/streamio-scanner-stdin-final.json -j 1 --dist web/dist`: `total=5`, `passed=5`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-scanner-streamio-final.json -j 1 --dist web/dist`: `total=14`, `passed=6`, `failed=8`。残りは StreamWriter 側の raw header/buffer load で、`ISS-20260429T123427866Z-STREAMIO-WRITER-RAW-BUFFER-LOADS-FAI-77152BD3` として分離した。
- `origin/main` の `78f310e` 同期後に `trunk build` と `node nodesrc/test_stdlib_streamio_scanner_boundary.js` は passed。
- 同期後の `tests/stdlib/stdin.n.md` は `total=5`, `failed=5`。失敗は `stdio_finish_read_buffer` / `string_from_mem_unchecked_result` / `fs_open_with_flags` / `fs_read_fd_bytes` の ResourceIR owner/raw cell 問題で、scanner raw byte load の再発ではない。
- 同期後の `tests/stdlib/streamio.n.md` は `total=14`, `passed=2`, `failed=12`。失敗は StreamWriter / stdio / fs / string / alloc io 側の別問題で、scanner source policy は clean。
