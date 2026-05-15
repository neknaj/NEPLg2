---
id: ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686
title: "public alloc_ptr APIs still encode free obligation in MemPtr"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/core/mem/pointer/alloc.nepl; stdlib/core/mem.nepl; stdlib/std/fs/**/*.nepl; stdlib/std/stdio/**/*.nepl; nepl-core/src/resource"
---

# ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686: public alloc_ptr APIs still encode free obligation in MemPtr

## 概要

alloc_ptr/realloc_ptr/dealloc_ptr remain public checked APIs whose free obligation is represented by MemPtr<T>, even though Stage 6 now treats MemPtr<T> as a non-owning pointer projection.

## 対象

- `stdlib/core/mem/pointer/alloc.nepl; stdlib/core/mem.nepl; stdlib/std/fs/**/*.nepl; stdlib/std/stdio/**/*.nepl; nepl-core/src/resource`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は `MemPtr<T>` を non-owning pointer projection に固定し、free obligation owner を `OwnedRegion` / `OwnedBuffer` / compiler-issued token 側へ分離する方針である。
- `stdlib/core/mem/pointer/alloc.nepl` は `alloc_ptr<T> -> Result<MemPtr<T>, str>`、`realloc_ptr<T> -> Result<MemPtr<T>, str>`、`dealloc_ptr<T>(MemPtr<T>, i32)` を public API として提供している。
- `stdlib/core/mem.nepl` / `stdlib/core/mem/pointer.nepl` の doctest は ordinary safe import から `alloc_ptr` / `dealloc_ptr` を使う例を示しており、`MemPtr<T>` の表面 contract と free obligation carrier としての Resource IR summary が食い違っている。
- `stdlib/std/fs` / `stdlib/std/stdio` / `stdlib/std/env/cliarg/raw` などの scratch buffer 実装は、現状まだ `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を temporary owner API として使っているため、即時削除ではなく token boundary への段階移行が必要である。

## 問題

alloc_ptr/realloc_ptr/dealloc_ptr remain public checked APIs whose free obligation is represented by MemPtr<T>, even though Stage 6 now treats MemPtr<T> as a non-owning pointer projection.

## 影響

Safe source and stdlib scratch helpers can still model storage ownership as MemPtr<T>, forcing Resource IR to keep owner-summary special cases for a type whose surface contract says it is non-owning.

## 修正方針

Split pointer allocation into an internal raw/scratch owner boundary and a compiler-issued owner token API. Migrate public examples and stdlib scratch storage to RegionToken or later OwnedRegion/OwnedBuffer, then stop exposing MemPtr-returning allocation as a safe root API.

## 検証

Add compile-fail/user-facing regressions that ordinary safe source cannot obtain a free obligation owner as MemPtr<T>; keep focused stdlib scratch tests proving owner cleanup through the new token boundary.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / mem / string 静的安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-16 Agent 1 調査メモ

`RegionToken<T>` から `MemPtr<T>` owner-like field を削除した後の残件として確認した。

現状の `MemPtr<T>` struct field baseline は 0 件だが、`alloc_ptr<T>` が public `MemPtr<T>` を返すため、struct layout ではなく関数 return API の形で free obligation owner が `MemPtr<T>` に残っている。これは `RegionToken<T>` field 修正とは別の根本問題である。

次の実装では、`alloc_ptr` 系を単に隠すのではなく、stdlib scratch buffer が必要とする「一時 owner」「raw syscall に渡す non-owning view」「cleanup failure の扱い」を typed token API に分ける。最終的には ordinary safe source が allocation owner を `MemPtr<T>` として取得できず、compiler が free obligation を owner token / initialized cell state として証明できる状態にする。

## 2026-05-16 Agent 1 部分対応メモ

子 issue [ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44](./ISS-20260515T170146857Z-CORE-MEM-POINTER-FACADE-RE-EXPORTS-L-4724AF44.md) で、`core/mem` / `mem/pointer` safe facade から `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` の re-export を外した。

これで `#import "core/mem" as *` だけでは `MemPtr<T>` owner API へ到達できない。`mem_ptr_add` は low-level alloc file から `pointer/view` へ分離し、owner API と non-owning view helper の責務を分けた。

ただし `#import "core/mem/pointer/alloc" as *` による direct import はまだ残っており、stdlib scratch 実装もこの低レベル境界に依存している。したがって親 issue は open のまま維持し、次段階では direct low-level alloc API を token / storage owner API へ置き換える。

## 2026-05-16 Agent 1 部分対応メモ 2

子 issue [ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42](./ISS-20260515T171022724Z-STDIO-PRINT-BYTE-SCRATCH-STILL-USES--2C662B42.md) で、`stdlib/std/stdio/write/byte.nepl` の 1 byte stdout scratch buffer を `RegionToken<u8>` owner 境界へ移した。

`print_byte` は direct `alloc_ptr` / `dealloc_ptr` / raw `store_u8` を使わず、`alloc_region`、`region_ptr`、checked `store_u8`、`dealloc_region` だけで一時 buffer を扱う。これにより public stdio helper の小さな scratch 境界から `MemPtr<u8>` free obligation carrier を削除した。

親 issue の残件は `std/stdio/write/fd.nepl`、`std/fs`、`std/env/cliarg/raw` などの syscall / ABI scratch 実装であり、raw syscall に iovec や C buffer を渡す箇所は個別に owner token と raw ABI view を分離していく。

## 2026-05-16 Agent 1 部分対応メモ 3

子 issue [ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA](./ISS-20260515T172241987Z-STDIO-FD-WRITE-SCRATCH-STILL-USES-ME-5A8C9CCA.md) で、`stdlib/std/stdio/write/fd.nepl` の WASI fd_write scratch を `RegionToken<u8>` owner 境界へ移した。

`stdio_write_fd_mem_result` は direct `alloc_ptr` / `dealloc_ptr` / `mem_ptr_addr` / raw `store_i32` / raw `load_i32` を持たず、`iov_region` / `nwritten_region` owner と non-owning view だけを扱う。iovec layout の raw address 操作は `std/stdio/raw.nepl` の `stdio_fd_write_from_result` へ閉じた。

これで `std/stdio/write` の stdout/stderr write 経路では `MemPtr<u8>` を free obligation carrier として扱わない。親 issue の残件は `std/fs` / `std/env/cliarg/raw` / `std/stdio/read` など、まだ direct low-level allocation owner を必要としている raw-backed boundary に絞られる。

## 2026-05-16 Agent 1 部分対応メモ 4

子 issue [ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719](./ISS-20260515T173402735Z-STDIO-READ-BYTES-STILL-USES-MEMPTR-O-571DB719.md) で、`stdlib/std/stdio/read` の `read_all` / `read_line` 経路を `RegionToken<u8>` owner 境界へ移す。

この対応では `stdio_read_all_bytes_result` の growable buffer、`stdio_read_line_result` の固定長 buffer、fd_read 用 iovec/nread scratch を direct `alloc_ptr` / `dealloc_ptr` / `realloc_ptr` から外し、`RegionToken<u8>` と non-owning `MemPtr<u8>` view に分離する。read 側の移行が完了すれば、`std/stdio` の主要 read/write 経路では `MemPtr<u8>` を free obligation carrier として扱わない。

## 2026-05-16 Agent 1 部分対応メモ 5

子 issue [ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B](./ISS-20260515T181501164Z-STD-FS-FD-WRITE-SCRATCH-STILL-USES-M-42F15E1B.md) で、`stdlib/std/fs/write/fd.nepl` の fd_write iovec / nwritten scratch を `RegionToken<u8>` owner 境界へ移した。

`fs_write_fd_mem_result` は direct `alloc_ptr` / `dealloc_ptr` を持たず、`iov_region` / `nwritten_region` owner と `region_ptr` から得た non-owning view だけを扱う。raw ABI layout の store/load は引き続き `std/fs/raw/fd_io.nepl` の `fs_fd_write_from_result` に閉じる。親 issue の残件は `std/fs/read/fd.nepl`、`std/fs/fd.nepl`、`std/fs/stat.nepl`、`std/fs/dir/read_fd.nepl`、`std/env/cliarg/raw.nepl` などの raw-backed boundary に残る direct allocation owner である。
