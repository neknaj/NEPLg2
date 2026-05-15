---
id: ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535
title: "read text doctests expose string_from_mem_unchecked_result owner leak"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource; stdlib/alloc/io/bytebuf.nepl; stdlib/alloc/string/storage.nepl; stdlib/std/stdio/read/text.nepl"
---

# ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535: read text doctests expose string_from_mem_unchecked_result owner leak

## 概要

After stdio read buffers were moved to RegionToken owners, focused read/text doctests still fail at compile time with resource.owner.maybe_leak in string_from_mem_unchecked_result. The read/buffer and read/bytes focused doctests pass, so the remaining failure is the ByteBuf-to-str/string constructor owner transfer boundary rather than the stdio read scratch owner itself.

## 対象

- `nepl-core/src/resource; stdlib/alloc/io/bytebuf.nepl; stdlib/alloc/string/storage.nepl; stdlib/std/stdio/read/text.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/std/stdio/read/text.nepl --no-tree -o tmp/agent1-read-text-string-owner-after-storage-boundary.json -j 1 --dist web/dist --assert-io` で、修正前は `string_from_mem_unchecked_result` 経由の `resource.owner.maybe_leak` が再現していた。
- 同じ stdio read 系でも `read/buffer` と `read/bytes` は通っていたため、read scratch owner ではなく `ByteBuf` から `str` へ確定する `string_finish` の所有権境界が原因だった。
- `RegionToken<T>` は既に `raw: i32, size: i32` layout へ移行済みだったが、`string_finish` は最後の確定境界で `get region "raw"` を一度 `MemPtr<u8>` として包み、`string_finish_base` 経由で `str` に変換していた。この stale wrapper により、Resource IR からは `RegionToken.raw` の free obligation が直接 `str` へ移ったことを追跡しにくかった。

## 問題

After stdio read buffers were moved to RegionToken owners, focused read/text doctests still fail at compile time with resource.owner.maybe_leak in string_from_mem_unchecked_result. The read/buffer and read/bytes focused doctests pass, so the remaining failure is the ByteBuf-to-str/string constructor owner transfer boundary rather than the stdio read scratch owner itself.

## 影響

stdin read_all/read_line text validation remains blocked by a memory-safety diagnostic even when read-side scratch ownership is fixed. Weakening Resource IR would hide real leaks, so the string/ByteBuf owner transfer must be proven precisely.

## 修正方針

`io_bytebuf_to_str_result -> string_from_mem_unchecked_result -> string_finish` の owner flow を、`RegionToken.raw` layout 後の境界として修正する。`string_finish` は `RegionToken` の raw owner identity を直接取り出し、ヘッダ長を書いた同じ raw owner を `string_from_addr_unchecked` に渡して `str` へ移す。Resource IR の検査を弱めず、stdlib constructor boundary の stale `MemPtr` wrapper を除去する。

## 検証

- `node nodesrc/tests.js -i stdlib/std/stdio/read/text.nepl --no-tree -o tmp/agent1-read-text-string-owner-after-storage-boundary.json -j 1 --dist web/dist --assert-io`: 2 passed
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl -i stdlib/std/stdio/read/bytes.nepl -i stdlib/std/stdio/read/text.nepl --no-tree -o tmp/agent1-stdio-read-string-owner-final.json -j 1 --dist web/dist --assert-io`: 3 passed
- `node nodesrc/test_stdlib_string_storage_boundary.js`
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`
- `node nodesrc/test_stdlib_stdio_read_boundary.js`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_raw_owner_through_str_from_addr -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_string_from_mem_unchecked_result_transfer -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_source_after_string_from_mem_copy -- --nocapture`

`stdlib/alloc/io/bytebuf.nepl` を含めた広めの doctest では、既存の `from_i128_radix` / JSON quote 系 builder owner leak が残る。これは `ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB` および `ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA` に記録済みの別 issue であり、この read/text owner transfer の修正範囲からは分離する。
