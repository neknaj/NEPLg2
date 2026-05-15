---
id: ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42
title: "stdio print_byte scratch still uses MemPtr alloc owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: stdlib/std/stdio/write/byte.nepl
---

# ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42: stdio print_byte scratch still uses MemPtr alloc owner

## 概要

std/stdio/write/byte.nepl used alloc_ptr/dealloc_ptr directly for its one-byte stdout scratch buffer, so a public stdio helper still modeled temporary storage ownership as MemPtr<u8> even though Stage 6 defines MemPtr as a non-owning pointer view.

## 対象

- `stdlib/std/stdio/write/byte.nepl`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr<T>` を non-owning pointer view に固定し、free obligation owner を `RegionToken` / `OwnedRegion` / storage token 側へ分離する方針である。
- 親 issue [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md) は、direct `alloc_ptr` / `dealloc_ptr` 依存を stdlib scratch 境界から減らすことを Stage 6 残件として追跡している。
- `print_byte` は 1 byte の private scratch buffer だけを必要とするため、`MemPtr<u8>` を owner として受け取る必要はなく、`RegionToken<u8>` owner と `region_ptr` view に分離できる。

## 問題

std/stdio/write/byte.nepl used alloc_ptr/dealloc_ptr directly for its one-byte stdout scratch buffer, so a public stdio helper still modeled temporary storage ownership as MemPtr<u8> even though Stage 6 defines MemPtr as a non-owning pointer view.

## 影響

The public alloc_ptr migration could not converge while small stdlib scratch helpers still required MemPtr-returning allocation APIs. Resource IR also had to keep treating MemPtr as a possible free-obligation carrier for this boundary.

## 修正方針

Move print_byte scratch ownership to RegionToken<u8>, derive only a non-owning MemPtr<u8> view with region_ptr, write via checked store_u8, and consume the owner with dealloc_region on all paths.

## 検証

Run the stdio boundary source policy and focused stdio write doctest suite.

## 対応結果

- `stdlib/std/stdio/write/byte.nepl` から direct `core/mem/pointer/alloc` / `core/mem/internal` / `core/mem/raw` import を削除した。
- `print_byte` は `alloc_region<u8>` で scratch owner token を確保し、`region_ptr &region` から得た non-owning `MemPtr<u8>` view へ checked `store_u8` で 1 byte を書く。
- 書き込み成功時も失敗時も `dealloc_region<u8> region` で free obligation を閉じる。`MemPtr` は `stdio_write_mem` へ渡す non-owning view としてだけ使う。
- unit 互換 API なので cleanup error は呼び出し側へ返せないが、unsafe helper には落とさない。`print_byte_result` のような fallible API が必要になった場合は別 issue として設計する。

## 回帰テスト

- `node nodesrc/test_stdlib_stdio_read_boundary.js`
- `node nodesrc/test_stdlib_no_unsafe_helpers.js`
- `node nodesrc/tests.js -i stdlib/std/stdio/write/byte.nepl -i stdlib/std/stdio/write.nepl --no-tree -o tmp/agent1-stdio-byte-region-scratch-doctests.json -j 1 --dist web/dist --assert-io`
