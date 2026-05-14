---
id: ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02
title: "Vec storage MemPtr helper exposes lower-level storage state"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/vec/storage/view.nepl, stdlib/alloc/collections/vec/access/data.nepl"
---

# ISS-20260514T161819706Z-VEC-STORAGE-MEMPTR-HELPER-EXPOSES-LO-A9C5BC02: Vec storage MemPtr helper exposes lower-level storage state

## 概要

vec_storage_mem_ptr<T> is public and takes VecStorageState plus a borrowed RegionToken<T>. Even after data_ptr removal, ordinary callers can depend on lower-level storage-state projection instead of using the Vec owner observer boundary.

## 対象

- `stdlib/alloc/collections/vec/storage/view.nepl, stdlib/alloc/collections/vec/access/data.nepl`

## 根拠

- `vec_storage_mem_ptr<T>` は `VecStorageState` と `&RegionToken<T>` を直接受ける public helper だった。
- 実装上の利用箇所は `data_mem_ptr<T>(&Vec<T>)` だけであり、public caller に lower-level storage state pieces を渡させる必要はない。
- `VecStorageState::Empty` / `Owned` の match は public storage helper ではなく、`&Vec<T>` を受ける observer boundary が所有すべき責務である。

## 問題

vec_storage_mem_ptr<T> is public and takes VecStorageState plus a borrowed RegionToken<T>. Even after data_ptr removal, ordinary callers can depend on lower-level storage-state projection instead of using the Vec owner observer boundary.

## 影響

Stage 6 still exposes an implementation-shaped storage helper. Future OwnedBuffer/borrow-projection work must preserve a public helper that accepts internal state pieces, making the public API surface wider than the actual safe Vec observer contract.

## 修正方針

Remove vec_storage_mem_ptr from the public storage facade and inline the storage-state match into data_mem_ptr(&Vec<T>). Keep raw pointer conversion inside the Vec access/data boundary, update source policy to forbid reintroducing vec_storage_mem_ptr, and keep Copy-only regression coverage.

## 検証

Run Vec source policy checks, focused Vec access/storage doctests, Vec/KP focused doctests, issue check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 修正内容

- `vec_storage_mem_ptr<T>` を削除し、lower-level `(VecStorageState, &RegionToken<T>) -> MemPtr<T>` helper を public API から外した。
- `data_mem_ptr<T>(&Vec<T>)` が `VecStorageState` を直接 match し、`Empty` では `mem_ptr_wrap 0`、`Owned` では `region_ptr` 由来の non-owning view を返す。
- `storage/view.nepl` は empty `Vec` construction だけを所有する file に戻した。
- source policy は `vec_storage_mem_ptr` 再導入を拒否し、`data_mem_ptr` が storage-state projection を所有することを監視する。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/alloc/collections/vec/storage/view.nepl -i stdlib/tests/vec.n.md -i stdlib/kp/kpsearch.nepl --no-tree -o tmp/agent1-vec-storage-helper-boundary.json -j 1 --dist web/dist --assert-io`: total=11, passed=11
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
