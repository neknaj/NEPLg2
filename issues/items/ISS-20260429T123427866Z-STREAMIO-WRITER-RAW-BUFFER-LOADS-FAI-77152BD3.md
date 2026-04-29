---
id: ISS-20260429T123427866Z-STREAMIO-WRITER-RAW-BUFFER-LOADS-FAI-77152BD3
title: "streamio writer raw buffer loads fail RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
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

## 検証

node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-writer-rawmemory-after.json -j 1 --dist web/dist should pass.
