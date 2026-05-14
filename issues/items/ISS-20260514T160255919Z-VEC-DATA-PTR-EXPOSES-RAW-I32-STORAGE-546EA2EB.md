---
id: ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB
title: "Vec and KP expose raw i32 storage address public APIs"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/vec/access/data.nepl, stdlib/kp/kpsearch.nepl"
---

# ISS-20260514T160255919Z-VEC-DATA-PTR-EXPOSES-RAW-I32-STORAGE-546EA2EB: Vec and KP expose raw i32 storage address public APIs

## 概要

Vec.data_ptr<T> publicly converts the borrowed storage view into a raw i32 address. KP search also exposed public raw i32 pointer helpers for lower/upper bound and unique. Copy-gating prevents non-Copy payload leaks, but ordinary callers can still depend on raw address identity instead of a typed MemPtr view or safe Vec APIs, keeping the raw-memory-backed boundary in the public API.

## 対象

- `stdlib/alloc/collections/vec/access/data.nepl, stdlib/kp/kpsearch.nepl`

## 根拠

- `data_ptr<T>(&Vec<T>) -> i32` は `RegionToken<T>` 由来の borrowed `MemPtr<T>` view を raw address へ戻して公開していた。
- KP search の `lower_bound_i32` / `upper_bound_i32` / `contains_i32` / `count_equal_range_i32` / `unique_sorted_i32` は raw `i32` pointer と length を public API として受けていた。
- `MemPtr = non-owning pointer` / `OwnedRegion = free obligation owner` の Stage 6 方針では、raw address は raw-memory-boundary implementation point でだけ明示的に使い、ordinary source の公開 API へは出さない。

## 問題

Vec.data_ptr<T> publicly converts the borrowed storage view into a raw i32 address. KP search also exposed public raw i32 pointer helpers for lower/upper bound and unique. Copy-gating prevents non-Copy payload leaks, but ordinary callers can still depend on raw address identity instead of a typed MemPtr view or safe Vec APIs, keeping the raw-memory-backed boundary in the public API.

## 影響

Resource IR and future borrow/lifetime checks must preserve a legacy raw address observer surface, and stdlib callers can bypass typed projection discipline. This conflicts with the Stage 6 MemPtr=non-owning-view direction and makes public API migration harder.

## 修正方針

Remove the public data_ptr API instead of keeping a compatibility alias. Move stdlib call sites to data_mem_ptr<T>(&Vec<T>) plus an explicit mem_ptr_addr only at raw-memory-boundary implementation points. Keep KP raw search helpers private and expose Vec-owner wrappers as the public API. Update source policy/doctests to forbid reintroducing data_ptr or public KP raw pointer helpers.

## 検証

Run Vec source policy checks, Vec doctests focused on stdlib/tests/vec.n.md and kpsearch, issue check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 修正内容

- `Vec.data_ptr<T>` を削除し、互換 alias は残さない。
- `Vec.data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` は `.T: Copy` 制約の typed view observer として残し、raw address が必要な実装箇所だけで明示的に `mem_ptr_addr` を呼ぶ。
- `kpsearch` の raw `i32` pointer helper を private にし、公開 API は `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` / `unique_sorted_vec_i32` に揃えた。
- `kpsearch` の doctest は raw buffer を ordinary source で構築する例から、`Vec<i32>` owner を渡す例へ変更した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` / `nodesrc/test_stdlib_vec_borrowed_observers.js` / `nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js` で退行を監視する。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/tests/vec.n.md -i stdlib/kp/kpsearch.nepl --no-tree -o tmp/agent1-vec-data-ptr-removal.json -j 1 --dist web/dist --assert-io`: total=11, passed=11
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kp-after-raw-api-private.json -j 1 --dist web/dist --assert-io`: total=7, passed=7
