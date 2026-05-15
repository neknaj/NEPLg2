---
id: ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535
title: "read text doctests expose string_from_mem_unchecked_result owner leak"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource; stdlib/alloc/io/bytebuf.nepl; stdlib/alloc/string/storage.nepl; stdlib/std/stdio/read/text.nepl"
---

# ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535: read text doctests expose string_from_mem_unchecked_result owner leak

## 概要

After stdio read buffers were moved to RegionToken owners, focused read/text doctests still fail at compile time with resource.owner.maybe_leak in string_from_mem_unchecked_result. The read/buffer and read/bytes focused doctests pass, so the remaining failure is the ByteBuf-to-str/string constructor owner transfer boundary rather than the stdio read scratch owner itself.

## 対象

- `nepl-core/src/resource; stdlib/alloc/io/bytebuf.nepl; stdlib/alloc/string/storage.nepl; stdlib/std/stdio/read/text.nepl`

## 根拠

- 未記入

## 問題

After stdio read buffers were moved to RegionToken owners, focused read/text doctests still fail at compile time with resource.owner.maybe_leak in string_from_mem_unchecked_result. The read/buffer and read/bytes focused doctests pass, so the remaining failure is the ByteBuf-to-str/string constructor owner transfer boundary rather than the stdio read scratch owner itself.

## 影響

stdin read_all/read_line text validation remains blocked by a memory-safety diagnostic even when read-side scratch ownership is fixed. Weakening Resource IR would hide real leaks, so the string/ByteBuf owner transfer must be proven precisely.

## 修正方針

Trace io_bytebuf_to_str_result -> string_from_mem_unchecked_result -> string_finish owner flow after RegionToken.raw changes. Fix Resource IR summaries or the stdlib constructor boundary so the output region owner is transferred to Result::Ok str and input ByteBuf cleanup remains explicit on all paths.

## 検証

node nodesrc/tests.js -i stdlib/std/stdio/read/text.nepl --no-tree -o tmp/read-text-string-owner.json -j 1 --dist web/dist --assert-io; focused Resource IR regression for string_from_mem_unchecked_result through ByteBuf-to-str
