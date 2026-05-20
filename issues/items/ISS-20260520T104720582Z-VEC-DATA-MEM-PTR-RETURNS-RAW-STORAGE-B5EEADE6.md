---
id: ISS-20260520T104720582Z-VEC-DATA-MEM-PTR-RETURNS-RAW-STORAGE-B5EEADE6
title: "Vec data_mem_ptr returns raw storage view before proving OwnedBuffer invariant"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/access/data.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260520T104720582Z-VEC-DATA-MEM-PTR-RETURNS-RAW-STORAGE-B5EEADE6: Vec data_mem_ptr returns raw storage view before proving OwnedBuffer invariant

## 概要

data_mem_ptr<T>(&Vec<T>) matches VecStorage and returns region_ptr for Owned storage without proving len / initialized_len / cap / storage correlation. Callers now guard raw load/store, but the public typed data view observer can still derive a raw view from malformed Vec metadata.

## 対象

- `stdlib/alloc/collections/vec/access/data.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `data_mem_ptr<T>(&Vec<T>)` は `VecStorage::Owned(region)` を見れば `region_ptr region` を返していた。
- 修正前は `len` / `initialized_len` / `cap` / storage variant の相関を確認していないため、malformed `OwnedBuffer<T>` から non-owning raw storage view を導出できた。
- raw load/store caller 側は invariant guard を持つようになったが、typed raw view observer 自体が guard の外に残ると、将来の stdlib / self-host 実装が同じ穴を再導入しやすい。

## 問題

data_mem_ptr<T>(&Vec<T>) matches VecStorage and returns region_ptr for Owned storage without proving len / initialized_len / cap / storage correlation. Callers now guard raw load/store, but the public typed data view observer can still derive a raw view from malformed Vec metadata.

## 影響

Stage D should make raw view derivation itself depend on the same Vec invariant as raw element traversal. Leaving data_mem_ptr outside the invariant boundary lets future stdlib or self-host code regain raw storage identity before proving the collection state.

## 修正方針

Guard data_mem_ptr with vec_current_copy_invariant<T>. Return a null typed view for invalid or Empty state, and keep actual RegionToken projection only for valid Owned storage.

## 検証

Add source policy coverage for data_mem_ptr invariant guarding, run Vec access doctests, vec source policy, and issues check.

## 2026-05-20 Agent 1 修正

`data_mem_ptr<T>(&Vec<T>)` が `OwnedBuffer<T>` の current Copy-only invariant を確認してから `RegionToken<T>` の `region_ptr` projection を返すようにした。

invalid owner aggregate または `VecStorage::Empty` では `mem_ptr_wrap 0` を返す。これにより public typed raw view observer は、raw load/store caller と同じ invariant boundary を共有する。

回帰テスト:

- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、`data_mem_ptr` が `vec_buffer_current_copy_invariant<T>` を `region_ptr` より前に通ることを検査する source policy を追加した。

focused verification:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree --dist web/dist -o tmp/agent1-vec-data-view-invariant-2.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree --dist web/dist -o tmp/agent1-vec-root-data-view-invariant.json -j 1 --assert-io`: 3/3 passed

## 2026-05-20 Agent 1 follow-up

この issue の修正で入れた null `MemPtr` sentinel は、後続の `ISS-20260520T113031972Z-VEC-DATA-VIEW-COLLAPSES-INVALID-INVA-D6378EBA` で破棄した。

現在の設計では `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を残さず、`data_mem_view<T>(&Vec<T>) -> VecDataView<T>` が `Empty | Data(MemPtr<T>) | Invalid(VecCopyInvariantInvalid)` を返す。invalid owner aggregate を empty storage と同じ pointer value に潰さず、typed refutation evidence を caller の `match` に残す。
