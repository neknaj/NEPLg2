---
id: ISS-20260517T032727149Z-CORE-MEM-RAW-DOCTESTS-CALL-RAW-APIS--057347FB
title: "core/mem raw doctests call raw APIs from ordinary source"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "stdlib/core/mem/allocator.nepl, stdlib/core/mem/raw.nepl"
---

# ISS-20260517T032727149Z-CORE-MEM-RAW-DOCTESTS-CALL-RAW-APIS--057347FB: core/mem raw doctests call raw APIs from ordinary source

## 概要

core/mem allocator/raw doctests are compiled as ordinary user sources but still call alloc_raw, mem_size, memset_u8, and fill_i32 as successful examples.

## 対象

- `stdlib/core/mem/allocator.nepl, stdlib/core/mem/raw.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/core/mem --no-tree -o tmp/core-mem-raw-doctest-boundary-before.json -j 1` で 33 件中 4 件が失敗した。
- 失敗した doctest は `allocator.nepl::doctest#1`、`raw.nepl::doctest#1/#2/#3` で、いずれも ordinary doctest entry から raw memory operation を呼ぶ形だった。
- 現在の Stage 6 設計では、`core/mem/raw` と `core/mem/allocator` の raw operation は compiler-owned raw-memory boundary 内だけで使う。doctest entry はその境界ではないため、拒否されるのが正しい。

## 問題

core/mem allocator/raw doctests are compiled as ordinary user sources but still call alloc_raw, mem_size, memset_u8, and fill_i32 as successful examples.

## 影響

Stage 6 raw-memory boundary checks correctly reject those doctests, so focused core/mem doctest runs fail unless the docs model the public/internal boundary accurately.

## 修正方針

Convert raw API doctests that run as ordinary sources into compile_fail boundary fixtures, and keep executable examples on safe RegionToken/MemPtr wrappers.

## 検証

- `node nodesrc/tests.js -i stdlib/core/mem --no-tree -o tmp/core-mem-raw-doctest-boundary-after.json -j 1`: 33 passed。
- `node nodesrc/issues.js check --dir issues`: passed。
- `node nodesrc/test_doctest_diag_code_metadata.js`: passed。
