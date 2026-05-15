---
id: ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719
title: "stdio read paths still use MemPtr owner buffers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: stdlib/std/stdio/read
---

# ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719: stdio read paths still use MemPtr owner buffers

## 概要

stdio_read_all_bytes_result and stdio_read_line_result allocate their growable/fixed input buffers and fd_read scratch buffers through alloc_ptr/realloc_ptr/dealloc_ptr, so read-side storage ownership is still carried by MemPtr<u8> even though Stage 6 defines MemPtr as a non-owning pointer view.

## 対象

- `stdlib/std/stdio/read`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr<T>` を non-owning pointer view とし、free obligation owner を `RegionToken` / OwnedRegion / storage wrapper 側へ分離する方針である。
- 親 issue [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md) は、stdlib scratch buffer に残る direct `alloc_ptr` / `dealloc_ptr` 依存を Stage 6 の残件として追跡している。
- `std/stdio/write` は `RegionToken<u8>` scratch と raw ABI helper 境界へ移行済みであり、read 側だけが同じ fd scratch / buffer owner を `MemPtr<u8>` owner として残していた。

## 問題

stdio_read_all_bytes_result and stdio_read_line_result allocate their growable/fixed input buffers and fd_read scratch buffers through alloc_ptr/realloc_ptr/dealloc_ptr, so read-side storage ownership is still carried by MemPtr<u8> even though Stage 6 defines MemPtr as a non-owning pointer view.

## 影響

The public alloc_ptr migration remains blocked by central stdin read paths. Resource IR must keep accepting stdio paths where MemPtr acts as a free-obligation owner, weakening the owner/view split required for memory-safety proofs.

## 修正方針

Move stdin read buffer ownership to RegionToken<u8>, derive MemPtr<u8> only as non-owning views with region_ptr, route read_all grow/shrink through realloc_region_bytes_keep, consume scratch/main owners with dealloc_region, and keep fd_read raw ABI layout in a narrow helper boundary.

## 修正内容

- `stdio_read_all_bytes_result` の main buffer / iovec / nread scratch を `alloc_region<u8>` で確保し、loop 内の書き込み先は `region_ptr &buf_region` から得る non-owning `MemPtr<u8>` view にした。
- read_all の grow は `realloc_region_bytes_keep<u8>` を使い、成功時は新 `RegionToken<u8>`、失敗時は `RegionReallocError` から旧 token を戻して cleanup できる形にした。容量 overflow / allocator payload 超過は `CapacityExceeded` として分ける。
- `stdio_finish_read_buffer` / `stdio_discard_read_buffer` は `MemPtr<u8>` + cap ではなく `RegionToken<u8>` owner を消費する API に変更した。exact-size `ByteBuf` 化も `io_bytebuf_finish_region` に委譲する。
- `stdio_read_line_result` も固定長 buffer / iovec / nread scratch を `RegionToken<u8>` owner へ移し、1 byte 読込後の検査は checked `store_u8` / `load_u8` で行うようにした。
- read 側の cleanup branch から `#intrinsic "unreachable"` を削除し、unsafe helper を error path の穴埋めとして使わない形にした。
- `nodesrc/test_stdlib_stdio_read_boundary.js` を更新し、read helpers が `RegionToken<u8>` owner を消費すること、read_all/read_line が low-level `alloc_ptr` / `dealloc_ptr` / raw layout 操作を再導入しないことを監視する。

## 検証

node nodesrc/test_stdlib_stdio_read_boundary.js; node nodesrc/test_stdlib_no_unsafe_helpers.js; focused doctests for stdlib/std/stdio/read/bytes.nepl, stdlib/std/stdio/read/text.nepl, and stdlib/std/stdio/read/buffer.nepl; node nodesrc/issues.js check; git diff --check

## 検証結果

- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/test_stdlib_no_unsafe_helpers.js`: passed
- `node nodesrc/test_stdlib_documentation_contract.js`: baseline ok
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/read/buffer.nepl -i stdlib/std/stdio/read/bytes.nepl --no-tree -o tmp/agent1-stdio-read-region-owner-buffer-bytes.json -j 1 --dist web/dist --assert-io`: 1 passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

`stdlib/std/stdio/read/text.nepl` の doctest は `string_from_mem_unchecked_result` の `resource.owner.maybe_leak` で失敗した。read/buffer + read/bytes だけの focused run は通っているため、この failure は stdio read scratch owner ではなく ByteBuf-to-str / string constructor owner transfer の残件として [ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535](./ISS-20260515T174246376Z-READ-TEXT-DOCTESTS-EXPOSE-STRING-FRO-F2CEA535.md) に分離した。
