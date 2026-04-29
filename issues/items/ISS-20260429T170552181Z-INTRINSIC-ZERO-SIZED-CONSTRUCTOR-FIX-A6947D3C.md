---
id: ISS-20260429T170552181Z-INTRINSIC-ZERO-SIZED-CONSTRUCTOR-FIX-A6947D3C
title: "intrinsic zero-sized constructor fixture leaks raw allocations"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: tests/compiler/intrinsic.n.md
---

# ISS-20260429T170552181Z-INTRINSIC-ZERO-SIZED-CONSTRUCTOR-FIX-A6947D3C: intrinsic zero-sized constructor fixture leaks raw allocations

## 概要

tests/compiler/intrinsic.n.md::intrinsic_zero_sized_struct_constructor_keeps_heap_pointer allocates p0 and p1 with alloc_raw but never deallocates them. With the Resource IR owner gate enabled, the fixture fails with owner obligation leaks for p0 and p1.

## 対象

- `tests/compiler/intrinsic.n.md`

## 根拠

- `intrinsic_zero_sized_struct_constructor_keeps_heap_pointer` は `alloc_raw 16` を 2 回呼び、`p0` と `p1` を作る。
- fixture は `load_i32 p0` と `gt p1 p0` で検証値を作るが、その後 `p0` / `p1` を `dealloc_raw` していなかった。
- Resource IR owner gate は raw allocation owner の未解放を正しく `resource.raw.ownership_violation` として報告していた。

## 問題

tests/compiler/intrinsic.n.md::intrinsic_zero_sized_struct_constructor_keeps_heap_pointer allocates p0 and p1 with alloc_raw but never deallocates them. With the Resource IR owner gate enabled, the fixture fails with owner obligation leaks for p0 and p1.

## 影響

The compiler test suite reports a Resource IR ownership failure for a fixture bug, hiding real owner-checker regressions and keeping tests/compiler from serving as a clean regression gate.

## 修正方針

Keep the heap-pointer preservation assertions, but store the observed booleans before freeing both raw allocations with dealloc_raw. The test should prove the same behavior while satisfying raw owner obligations.

## 修正内容

- `kept` / `moved` を計算した後、結果判定前に `dealloc_raw p0 16` と `dealloc_raw p1 16` を追加した。
- heap pointer preservation の検証内容は維持しつつ、raw allocation owner obligation を fixture 側で満たすようにした。

## 検証

trunk build; node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/intrinsic-raw-dealloc-fixture.json -j 1 --dist web/dist; node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-intrinsic-raw-dealloc.json -j 4 --dist web/dist

- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/intrinsic-raw-dealloc-fixture.json -j 1 --dist web/dist`: total=8, passed=8
- `node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-intrinsic-raw-dealloc.json -j 4 --dist web/dist`: total=649, passed=637, failed=12。`intrinsic.n.md::doctest#6` は解消し、残りは既存の ResourceIR owner obligation 系。
