---
id: ISS-20260523T051715144Z-VEC-NON-COPY-SORT-NEEDS-BORROWED-COM-7B8AAE90
title: "Vec non-Copy sort needs borrowed comparison and slot swap lifecycle proof"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-23
updated: 2026-05-23
target: "stdlib/alloc/collections/vec/sort/**, nepl-core/src/resource/**"
---

# ISS-20260523T051715144Z-VEC-NON-COPY-SORT-NEEDS-BORROWED-COM-7B8AAE90: Vec non-Copy sort needs borrowed comparison and slot swap lifecycle proof

## 概要

Vec sort implementations remain Ord&Copy and use raw load/store/swap over MemPtr-backed views. Non-Copy payload sorting cannot be implemented by removing Copy bounds because comparison must borrow, and swaps must be represented as ownership-preserving move/replace lifecycle proof.

## 対象

- `stdlib/alloc/collections/vec/sort/**, nepl-core/src/resource/**`

## 根拠

- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の残件監査で、`Vec` sort はまだ Copy raw access / by-value comparison 前提に残っていることを確認した。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、slot state の move / borrow / replace / drop を typed enum と exhaustive match に載せる方針であり、sort だけ shallow raw swap を例外扱いできない。
- `stdlib/alloc/collections/vec/sort/quick.nepl`, `heap.nepl`, `common.nepl`, `sort/merge/api.nepl` は `Ord&Copy` と raw load/store/swap 系 helper に依存している。
- non-Copy sort は transform より難しく、2 slot の borrowed comparison、temporary move-out、replace-return-old または swap lifecycle proof、panic/failure 時 cleanup を同時に設計する必要があるため、transform engine とは別 issue として追跡する。

## 問題

Vec sort implementations remain Ord&Copy and use raw load/store/swap over MemPtr-backed views. Non-Copy payload sorting cannot be implemented by removing Copy bounds because comparison must borrow, and swaps must be represented as ownership-preserving move/replace lifecycle proof.

## 影響

Self-host collections with owning payloads cannot be sorted safely. A shallow byte swap or by-value comparison would break move/drop state and could hide double-drop or leak bugs.

## 修正方針

Create a design and implementation plan for borrowed comparison (&T,&T)->bool, temporary move-out or replace-return-old based slot swap, and Resource IR lifecycle proof for sorting. Implement only after the generic transform/move/drop machinery is stable.

## 検証

Add source policy coverage for sort remaining Copy-only until lifecycle proof exists, plus future Resource IR regressions for non-Copy borrowed comparison and slot swap correctness.
