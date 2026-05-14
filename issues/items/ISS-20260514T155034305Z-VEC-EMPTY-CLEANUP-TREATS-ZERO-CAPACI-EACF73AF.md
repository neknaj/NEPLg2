---
id: ISS-20260514T155034305Z-VEC-EMPTY-CLEANUP-TREATS-ZERO-CAPACI-EACF73AF
title: "Vec empty cleanup treats zero-capacity sentinel as owned storage"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/storage/cleanup.nepl, stdlib/alloc/collections/vec/mutation/cleanup.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T155034305Z-VEC-EMPTY-CLEANUP-TREATS-ZERO-CAPACI-EACF73AF: Vec empty cleanup treats zero-capacity sentinel as owned storage

## 概要

VecStorageState::Empty represents storage with no allocation, but Vec.free routes every Vec.region through vec_free_storage and dealloc_region. This keeps the zero-size RegionToken sentinel on the same owner-consuming cleanup path as real allocated storage.

## 対象

- `stdlib/alloc/collections/vec/storage/cleanup.nepl, stdlib/alloc/collections/vec/mutation/cleanup.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` は、`VecStorageState::Empty` を allocation なしの state とし、`OwnedBuffer<T>` へ移るまで `RegionToken<T>` は過渡 owner token として扱う方針を示している。
- 既存の `Vec.free` は `storage` を見ずに `region` を `vec_free_storage` へ渡し、`vec_free_storage` は常に `dealloc_region` を呼んでいた。

## 問題

VecStorageState::Empty represents storage with no allocation, but Vec.free routes every Vec.region through vec_free_storage and dealloc_region. This keeps the zero-size RegionToken sentinel on the same owner-consuming cleanup path as real allocated storage.

## 影響

The storage-state proof is weaker than the Stage 6 memory-safety design: empty Vec cleanup relies on dealloc_region/dealloc rejecting the null sentinel instead of proving through VecStorageState that no free obligation exists. That normalizes RegionToken sentinel ownership and delays the OwnedBuffer/storage-state split.

## 修正方針

Make vec_free_storage take VecStorageState plus RegionToken, match exhaustively, no-op for Empty, and consume dealloc_region only for Owned. Update Vec.free and source policy so regressions cannot reintroduce unconditional empty-sentinel deallocation.

## 検証

Fixed.

- `vec_free_storage<T>` は `(VecStorageState, RegionToken<T>)` を受け取り、`VecStorageState::Empty` では no-op、`VecStorageState::Owned` では `dealloc_region` を呼ぶ形にした。
- `Vec.free<T>` は `storage` を先に借用で読み、`storage` と `region` owner を cleanup helper へ渡す。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は、empty zero-size sentinel を `dealloc_region` へ流さないことを監視する source policy へ更新した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/agent1-vec-empty-cleanup.json -j 1 --dist web/dist --assert-io`: 3/3 passed
- `node nodesrc/issues.js check`: passed
