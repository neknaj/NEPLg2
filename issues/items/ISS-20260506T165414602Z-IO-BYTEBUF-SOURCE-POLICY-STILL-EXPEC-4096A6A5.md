---
id: ISS-20260506T165414602Z-IO-BYTEBUF-SOURCE-POLICY-STILL-EXPEC-4096A6A5
title: "io ByteBuf source policy still expects checked dealloc after raw scratch cleanup migration"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, stdlib/std/fs/raw.nepl"
---

# ISS-20260506T165414602Z-IO-BYTEBUF-SOURCE-POLICY-STILL-EXPEC-4096A6A5: io ByteBuf source policy still expects checked dealloc after raw scratch cleanup migration

## 概要

node nodesrc/run_source_policy_regressions.js --warn-only still reports nodesrc/test_stdlib_io_bytebuf_owner_boundary.js because the policy expects fs_finish_read_buffer to call dealloc_ptr<u8> before returning io_bytebuf_empty. The current implementation deliberately uses dealloc_raw mem_ptr_addr buf cap for private scratch cleanup after the Resource IR owner-summary work made direct raw cleanup the documented boundary.

## 対象

- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, stdlib/std/fs/raw.nepl`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` は `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` で `fs_finish_read_buffer must deallocate scratch storage before returning an empty ByteBuf` を warning として報告していた。
- `stdlib/std/fs/raw.nepl` の `fs_finish_read_buffer` は invalid length path、0-byte read path、shrink failure path で private scratch buffer を `dealloc_raw mem_ptr_addr buf cap` により閉じている。
- `ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` の修正方針では、compiler-owned private scratch storage は checked `dealloc_ptr` ではなく exact `dealloc_raw` で閉じる設計に統一済みである。

## 問題

node nodesrc/run_source_policy_regressions.js --warn-only still reports nodesrc/test_stdlib_io_bytebuf_owner_boundary.js because the policy expects fs_finish_read_buffer to call dealloc_ptr<u8> before returning io_bytebuf_empty. The current implementation deliberately uses dealloc_raw mem_ptr_addr buf cap for private scratch cleanup after the Resource IR owner-summary work made direct raw cleanup the documented boundary.

## 影響

The stale policy keeps source policy regressions in warn-only failure state and can mislead future work back toward checked dealloc_ptr for private scratch storage, reintroducing owner-summary ambiguity that the stricter memory-safety work already removed.

## 修正方針

Update the ByteBuf owner boundary source policy to require exact dealloc_raw cleanup for fs_finish_read_buffer private scratch storage, while still rejecting leaked scratch buffers and direct ByteBuf construction. Keep dealloc_ptr for user-visible checked cleanup APIs, not compiler-owned internal scratch buffers.

## 対応

- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` の `fs_finish_read_buffer` 0-byte read assertion を `dealloc_ptr<u8> buf cap` 期待から `dealloc_raw mem_ptr_addr buf cap` 期待へ更新した。
- 同じ policy に `dealloc_ptr<u8> buf cap` への退行禁止を追加し、private scratch cleanup が checked dealloc wrapper に戻らないよう固定した。
- 実装側 `stdlib/std/fs/raw.nepl` は現行設計どおりだったため変更していない。

## 検証

Run node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, node nodesrc/run_source_policy_regressions.js --warn-only, node nodesrc/issues.js check, and git diff --check.

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: all source-policy regressions passed; warning 0
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
