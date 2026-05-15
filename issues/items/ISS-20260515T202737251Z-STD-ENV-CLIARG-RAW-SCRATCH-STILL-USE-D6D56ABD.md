---
id: ISS-20260515T202737251Z-STD-ENV-CLIARG-RAW-SCRATCH-STILL-USE-D6D56ABD
title: "std/env cliarg raw scratch still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/env/cliarg/raw.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260515T202737251Z-STD-ENV-CLIARG-RAW-SCRATCH-STILL-USE-D6D56ABD: std/env cliarg raw scratch still uses MemPtr owner API

## 概要

std/env/cliarg/raw.nepl still allocates argc metadata, argv pointer arrays, argv buffers, LLVM cmdline C strings, and cmdline temp buffers through alloc_ptr/dealloc_ptr, so the argv raw boundary keeps MemPtr as the free-obligation owner.

## 対象

- `stdlib/std/env/cliarg/raw.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- `cliarg_count_result` は argc metadata scratch を `alloc_ptr<u8> 8` で確保し、`MemPtr<u8>` を deallocation handle として扱っていた。
- `cliarg_get_checked` は metadata、argv pointer array、argv byte buffer を `alloc_ptr<u8>` で確保し、raw ABI address と free obligation owner を同じ `MemPtr<u8>` に持たせていた。
- LLVM fallback の `__cli_copy_to_cstr` / `args_sizes_get` / `args_get` も C string scratch と cmdline temp buffer を `Result<MemPtr<u8>, i32>` / `alloc_ptr` / `dealloc_ptr` で扱っていた。
- Stage 6 の方針では `MemPtr<T>` は non-owning pointer view であり、scratch の free obligation owner は `RegionToken` / storage owner 側へ分離する必要がある。

## 問題

std/env/cliarg/raw.nepl still allocates argc metadata, argv pointer arrays, argv buffers, LLVM cmdline C strings, and cmdline temp buffers through alloc_ptr/dealloc_ptr, so the argv raw boundary keeps MemPtr as the free-obligation owner.

## 影響

The final std raw-backed input boundary remains an exception to Stage 6's MemPtr non-owning contract and keeps PUBLIC-ALLOC-PTR migration open after std/fs and std/stdio moved to RegionToken owners.

## 修正方針

Move raw cliarg scratch buffers to RegionToken<u8> owners, derive MemPtr views from region_ptr only for raw ABI calls and checked byte access, deallocate through dealloc_region, and extend cliarg source policy.

## 検証

Run cliarg source policy, focused cliarg doctests/contract tests, issues check, and diff whitespace check.

## 解決

2026-05-16 Agent 1 で解決。

- argc metadata、argv pointer array、argv byte buffer、LLVM cmdline C string、LLVM cmdline temp buffer を `RegionToken<u8>` owner に移した。
- raw ABI と checked byte access に必要な `MemPtr<u8>` は `region_ptr &..._region` から得る non-owning view に限定した。
- cleanup は `dealloc_region<u8>` で owner token を消費する形に揃えた。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` に、raw cliarg scratch が `RegionToken` owner を使い、`core/mem/pointer/alloc` と `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を再導入しない policy を追加した。
- 検証中に、負 index が raw slot 計算へ到達する問題を [ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB](./ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB.md) として分離した。
- `std/env/cliarg/cstr.nepl` doctest が ordinary source から `mem_ptr_add` / `store_u8` を使う stale fixture も [ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B](./ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B.md) として分離した。
