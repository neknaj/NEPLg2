---
id: ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA
title: "stdio fd_write scratch still uses MemPtr alloc owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/stdio/raw.nepl; stdlib/std/stdio/write/fd.nepl"
---

# ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA: stdio fd_write scratch still uses MemPtr alloc owner

## 概要

std/stdio/write/fd.nepl allocated WASI iovec and nwritten scratch buffers with alloc_ptr/dealloc_ptr, so the fd write loop still modeled private scratch ownership as MemPtr<u8> instead of a separate owner token.

## 対象

- `stdlib/std/stdio/raw.nepl; stdlib/std/stdio/write/fd.nepl`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr<T>` を non-owning pointer view に固定し、scratch storage の free obligation を token / storage owner に分離する方針である。
- 親 issue [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md) は、stdlib scratch 実装に残る direct `alloc_ptr` / `dealloc_ptr` 依存を移行対象としている。
- `stdio_write_fd_mem_result` は stdout / stderr / streamio の中心的な fd write 経路なので、ここに `MemPtr<u8>` owner が残ると public alloc pointer migration の収束を妨げる。

## 問題

std/stdio/write/fd.nepl allocated WASI iovec and nwritten scratch buffers with alloc_ptr/dealloc_ptr, so the fd write loop still modeled private scratch ownership as MemPtr<u8> instead of a separate owner token.

## 影響

The public alloc_ptr migration remained blocked by a central stdio write path, and raw ABI layout operations were mixed into the partial-write loop instead of staying in a raw boundary helper.

## 修正方針

Move iovec and nwritten ownership to RegionToken<u8>, derive non-owning MemPtr views with region_ptr, add a stdio/raw helper that performs fd_write ABI layout initialization and nwritten load, and consume scratch owners with dealloc_region on all paths.

## 検証

Run stdio boundary/no-unsafe source policies and focused stdio fd/byte/stderr doctests.

## 対応結果

- `stdio_write_fd_mem_result` の `iov` / `nwritten` scratch allocation を `alloc_region<u8>` / `dealloc_region<u8>` に移した。
- raw address extraction、iovec layout store、`nwritten` load は新規 `stdio_fd_write_from_result` に閉じ、`std/stdio/write/fd.nepl` の partial-write loop は owner token と non-owning `MemPtr` view の管理に集中させた。
- `stdio_fd_write_from_result` には stdout と戻り byte 数を確認する doctest を追加した。allocation fixture は `match alloc_region` で書き、`unwrap_ok alloc_region` による provenance loss を避けた。
- cleanup Err branch は unsafe helper に落とさず、stdlib no-unsafe-helper policy を維持した。

## 回帰テスト

- `node nodesrc/test_stdlib_stdio_read_boundary.js`
- `node nodesrc/test_stdlib_no_unsafe_helpers.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib/std/stdio/raw.nepl -i stdlib/std/stdio/write/fd.nepl -i stdlib/std/stdio/write/byte.nepl --no-tree -o tmp/agent1-stdio-fd-region-scratch-min.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/stdio_result_stderr.n.md --no-tree -o tmp/agent1-stdio-fd-region-scratch-stderr.json -j 1 --dist web/dist --assert-io`

## 既知の別件

`tests/stdlib/stdout.n.md` を同時に含めると `from_i128_radix` / `from_u128_radix` / `concat_result` / `string_from_mem_unchecked_result` の `resource.owner.maybe_leak` で 3 件失敗する。これは stdio fd scratch の変更ではなく、既存の string owner summary / Stage 6 stdlib raw-memory-backed API 残件として扱う。
