---
id: ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B
title: "std/env cliarg cstr doctests use mem_ptr_add outside raw boundary"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/env/cliarg/cstr.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B: std/env cliarg cstr doctests use mem_ptr_add outside raw boundary

## 概要

std/env/cliarg/cstr.nepl doctests allocate RegionToken then write test bytes through mem_ptr_add/store_u8 in ordinary doctest source, which is rejected by Resource IR as resource.raw.memory_outside_boundary.

## 対象

- `stdlib/std/env/cliarg/cstr.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/env/cliarg/cstr.nepl` の doctest は `alloc_region` で byte buffer を作り、`mem_ptr_add` / `store_u8` で ordinary doctest source から NUL 終端 bytes を書いていた。
- Stage 6 では `mem_ptr_add` による pointer offset view と checked raw memory operation は raw-memory boundary 外で拒否されるため、doctest は `resource.raw.memory_outside_boundary` で compile failure になっていた。
- この失敗は静的検査を緩める理由ではなく、fixture が現在の raw boundary 方針に追従していない問題である。

## 問題

std/env/cliarg/cstr.nepl doctests allocate RegionToken then write test bytes through mem_ptr_add/store_u8 in ordinary doctest source, which is rejected by Resource IR as resource.raw.memory_outside_boundary.

## 影響

The cstr helper documentation examples are stale under the current Stage 6 raw boundary model and cannot serve as regression tests without weakening static checks.

## 修正方針

Rewrite cstr doctests to use safe public construction helpers or a compile_fail raw-boundary example, without granting raw memory authority to ordinary doctest code.

## 検証

Run cstr doctests and cliarg source policy.

## 解決

2026-05-16 Agent 1 で解決。

- cstr doctest は `alloc_region` / `store_u8` / `mem_ptr_add` を使わず、NUL を含む文字列 literal `"nep\0"` の `string_data_ptr` を C string pointer として渡す形に変更した。
- `cstr_len` / `cstr_to_str` の典型例は raw memory write を ordinary doctest source へ戻さずに pass する。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` に、cstr doctest が `string_data_ptr "nep\0"` を使い、`alloc_region` / `dealloc_region` / `store_u8` / `mem_ptr_add` / `unwrap_ok` を再導入しない検査を追加した。
