---
id: ISS-20260429T123427866Z-STREAMIO-WRITER-RAW-BUFFER-LOADS-FAI-77152BD3
title: "streamio writer raw buffer loads fail RawMemoryLoadCell gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/streamio.nepl, tests/stdlib/streamio.n.md"
---

# ISS-20260429T123427866Z-STREAMIO-WRITER-RAW-BUFFER-LOADS-FAI-77152BD3: streamio writer raw buffer loads fail RawMemoryLoadCell gate

## 概要

tests/stdlib/streamio.n.md still fails after scanner byte access is moved to MemPtr helpers. The remaining failures are in append_str_impl, append_bytebuf_impl, and stream_writer_load_header, where StreamWriter header/buffer raw loads are reported as RawMemoryLoadCell Uninit.

## 対象

- `stdlib/std/streamio.nepl, tests/stdlib/streamio.n.md`

## 根拠

- `tests/stdlib/streamio.n.md` は scanner 修正後も StreamWriter 系 doctest で `stream_writer_load_header` の `load_i32 p` を `RawMemoryLoadCell ... found Uninit` として報告する。
- 同期前の focused run では `append_str_impl` / `append_bytebuf_impl` の writer buffer byte access も残存 failure として確認した。
- StreamWriter header は `StreamWriterHeaderField` enum を持つが、load path は任意 raw pointer から `MemPtr<i32>`/raw load へ落ちており、header 初期化と後続 read の provenance が scanner と同様に切れている。

## 問題

tests/stdlib/streamio.n.md still fails after scanner byte access is moved to MemPtr helpers. The remaining failures are in append_str_impl, append_bytebuf_impl, and stream_writer_load_header, where StreamWriter header/buffer raw loads are reported as RawMemoryLoadCell Uninit.

## 影響

streamio writer doctests are not a clean regression target, and stdout/stderr buffered stream APIs cannot be trusted under strict raw memory initialization checking.

## 修正方針

Review StreamWriter header and append buffer boundaries. Do not weaken RawMemoryLoadCell; remove generic raw header loads and replace direct writer buffer raw loads with typed or local-initialized boundaries that preserve ResourceIR provenance.

## 修正内容

- `StreamWriter` の raw header 設計を破棄し、`buf` / `cap` / `write_len` / `target` を持つ非 Copy の owning struct に再設計した。
- `StreamWriterTargetKind` を `Clone` / `Copy` 対応 enum として保持し、flush 時は数値 code ではなく `match target` で stdout/stderr を分岐するようにした。
- `stream_writer_header_*` / `stream_writer_load_header*` / `stream_writer_store_header` / numeric target code helper を削除した。
- `stream_writer_new` は buffer owner を直接 `StreamWriter.buf` field へ入れて返し、header allocation と owner-in-raw-field transfer を不要にした。
- `drain_impl` / `reserve_impl` / `push_u8_impl` は struct field を `get_ref` で観測し、更新時だけ新しい `StreamWriter` を構築する形へ変更した。
- `append_str_impl` は `string_byte_at_unchecked` を使い、`string_data_ptr` からの直接 `load_u8` を削除した。
- `append_bytebuf_impl` は borrowed `ByteBuf` helper の `stream_writer_bytebuf_byte_at` を使い、copy 後に `io_bytebuf_free` する所有権契約を保った。
- `nodesrc/test_stdlib_streamio_writer_boundary.js` を追加し、raw header helper、numeric target code 分岐、append の直接 raw byte load の再導入を禁止した。

## 検証

- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-writer-owning-struct-after-sync.json -j 1 --dist web/dist`: `total=14`, `passed=5`, `failed=9`。StreamWriter raw header / raw buffer failure は消え、残りは既存の `io_bytebuf_from_str_result` / `stdio_finish_read_buffer` / `fs_open_with_flags` / `fs_read_fd_bytes` issue。
- `git diff --check`: passed
