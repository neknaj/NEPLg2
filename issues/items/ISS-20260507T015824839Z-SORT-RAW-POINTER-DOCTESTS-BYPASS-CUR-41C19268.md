---
id: ISS-20260507T015824839Z-SORT-RAW-POINTER-DOCTESTS-BYPASS-CUR-41C19268
title: "sort raw pointer doctests bypass current Resource IR owner and effect contract"
area: TEST
status: open
resolved: false
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: "tests/stdlib/sort.n.md; stdlib/alloc/collections/vec/sort/common.nepl; stdlib/alloc/collections/vec/sort/merge.nepl"
---

# ISS-20260507T015824839Z-SORT-RAW-POINTER-DOCTESTS-BYPASS-CUR-41C19268: sort raw pointer doctests bypass current Resource IR owner and effect contract

## 概要

tests/stdlib/sort.n.md doctest#18-#22 still construct raw MemPtr buffers directly, write through pure put_i32 helpers, and dealloc raw MemPtr storage without an owner token. With the current Resource IR/effect gate, doctest#18-#20 fail with effect.pure.calls_impure for store_i32 in a pure helper, while doctest#21/#22 fail with resource.owner.no_free_obligation / resource.cell.uninit around raw MemPtr deallocation and reads.

## 対象

- `tests/stdlib/sort.n.md; stdlib/alloc/collections/vec/sort/common.nepl; stdlib/alloc/collections/vec/sort/merge.nepl`

## 根拠

- 未記入

## 問題

tests/stdlib/sort.n.md doctest#18-#22 still construct raw MemPtr buffers directly, write through pure put_i32 helpers, and dealloc raw MemPtr storage without an owner token. With the current Resource IR/effect gate, doctest#18-#20 fail with effect.pure.calls_impure for store_i32 in a pure helper, while doctest#21/#22 fail with resource.owner.no_free_obligation / resource.cell.uninit around raw MemPtr deallocation and reads.

## 影響

The sort suite no longer cleanly validates sort_i32 under strict static checking. Leaving these fixtures stale can make CI failures look like compiler regressions or pressure future work to weaken raw memory effect, owner, or initialized-cell checks instead of fixing the test contract.

## 修正方針

Rewrite the raw pointer sort fixtures around the current memory model: use RegionToken or another owner-bearing allocation API, keep raw writes behind an explicit impure/internal boundary, prove initialization before raw reads, and release storage through the owner token. Do not weaken Resource IR owner/effect/cell checks.

## 検証

Run trunk build, then node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/sort-raw-pointer-fixtures-resource-contract.json -j 1 --dist web/dist; also keep nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js passing.
