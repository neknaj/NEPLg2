---
id: ISS-20260429T121949904Z-STREAMIO-SCANNER-RAW-BYTE-LOADS-FAIL-B5BB4131
title: "streamio scanner raw byte loads fail RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
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

- 未記入

## 問題

tests/stdlib/stdin.n.md doctest#5 fails in streamio scanner code under the RawMemoryLoadCell gate. The diagnostics point to stream_scanner_load_header_result, stream_scanner_skip_ws_header, and scan_i32_impl reading scanner header/buffer raw cells as Uninit.

## 影響

stdin scanner tests are not a clean regression target, and self-host / tutorial code that depends on StreamScanner cannot rely on streamio while strict raw memory initialization checking is enabled.

## 修正方針

Review std/streamio scanner header and byte access boundaries. Do not weaken RawMemoryLoadCell; replace generic raw header/byte loads or move initialization/read operations into boundaries that preserve ResourceIR initialization provenance.

## 検証

node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/streamio-scanner-rawmemory-after.json -j 1 --dist web/dist should pass, plus tests/stdlib/streamio.n.md when touching streamio.
