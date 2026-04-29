---
id: ISS-20260429T125126519Z-IO-BYTEBUF-FROM-STR-RESULT-LOSES-BYT-C0364ECE
title: "io_bytebuf_from_str_result loses ByteBuf owner under Resource IR"
area: stdlib
status: open
resolved: false
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

- 未記入

## 問題

After origin/main 78f310e, tests/stdlib/streamio.n.md reports io_bytebuf_from_str_result ConstructInput on ByteBuf out found Moved and out_raw owner may leak. The conversion allocates/copies bytes but the owner transfer into ByteBuf is not ResourceIR-clean.

## 影響

String-to-byte output helpers used by streamio and self-host IO cannot pass strict memory-safety checking. This blocks buffered output tests independently of StreamWriter header raw loads.

## 修正方針

Review alloc/io string-to-ByteBuf construction. Preserve the output allocation owner until the ByteBuf value is constructed exactly once, and ensure all allocation/copy failure paths free the output region or avoid creating an owner obligation.

## 検証

Run node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-bytebuf-owner-after.json -j 1 --dist web/dist and tests/stdlib/streamio.n.md focused fixtures.
