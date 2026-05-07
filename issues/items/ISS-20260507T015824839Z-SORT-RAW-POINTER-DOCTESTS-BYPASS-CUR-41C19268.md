---
id: ISS-20260507T015824839Z-SORT-RAW-POINTER-DOCTESTS-BYPASS-CUR-41C19268
title: "sort raw pointer doctests bypass current Resource IR owner and effect contract"
area: TEST
status: fixed
resolved: true
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

- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/sort-raw-pointer-fixtures-before.json -j 1 --dist web/dist` で、doctest#18-#22 が Resource IR/effect gate に拒否されることを確認した。
- 失敗内容は `effect.pure.calls_impure`、`resource.owner.no_free_obligation`、`resource.cell.uninit` であり、sort algorithm ではなく fixture の raw memory contract が現行仕様に合っていなかった。

## 問題

tests/stdlib/sort.n.md doctest#18-#22 still construct raw MemPtr buffers directly, write through pure put_i32 helpers, and dealloc raw MemPtr storage without an owner token. With the current Resource IR/effect gate, doctest#18-#20 fail with effect.pure.calls_impure for store_i32 in a pure helper, while doctest#21/#22 fail with resource.owner.no_free_obligation / resource.cell.uninit around raw MemPtr deallocation and reads.

## 影響

The sort suite no longer cleanly validates sort_i32 under strict static checking. Leaving these fixtures stale can make CI failures look like compiler regressions or pressure future work to weaken raw memory effect, owner, or initialized-cell checks instead of fixing the test contract.

## 修正方針

Rewrite the raw pointer sort fixtures around the current memory model: use RegionToken or another owner-bearing allocation API, keep raw writes behind an explicit impure/internal boundary, prove initialization before raw reads, and release storage through the owner token. Do not weaken Resource IR owner/effect/cell checks.

## 解決内容

`tests/stdlib/sort.n.md` の `sort_i32` pointer doctest 5 件を、直接 `alloc_ptr` / `store_i32` / `dealloc_raw` する fixture から、`Vec<i32>` が所有する初期化済み storage を使う fixture へ置き換えた。

- input 配列は `Vec` の `new` / `with_capacity` / `push` で構築する。
- `sort_i32` へは `data_mem_ptr<i32> &v` で得た `MemPtr<i32>` view を渡す。
- 検証は raw `load_i32` ではなく `Vec` の `get` で行い、initialized-cell proof を `Vec` 側に集約する。
- 解放は raw `dealloc_raw` ではなく `free<i32> v` で行い、owner obligation を `Vec` owner に残す。

これにより pointer API の `sort_i32` 自体は引き続き検証しつつ、fixture が raw memory checker を迂回しない形になった。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/sort-raw-pointer-fixtures-after-final.json -j 1 --dist web/dist`: total=22, passed=22
- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: passed
