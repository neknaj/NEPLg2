---
id: ISS-20260520T111714158Z-VEC-INVARIANT-DOES-NOT-PROVE-BACKING-46A3C801
title: "Vec invariant does not prove backing RegionToken extent"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: stdlib/alloc/collections/vec/invariant.nepl
---

# ISS-20260520T111714158Z-VEC-INVARIANT-DOES-NOT-PROVE-BACKING-46A3C801: Vec invariant does not prove backing RegionToken extent

## 概要

VecCopyInvariant validates len/initialized_len/cap/storage tag correlation but does not verify that VecStorage::Owned(region) has region_size == cap * size_of<T>. Raw push/sort/transform boundaries can therefore rely on cap metadata without a typed proof that the backing storage extent matches it.

## 対象

- `stdlib/alloc/collections/vec/invariant.nepl`

## 根拠

- `stdlib/alloc/collections/vec/invariant.nepl` の修正前 `VecCopyInvariant` は、`0 <= len == initialized_len <= cap` と `VecStorage::Empty | Owned` の tag/capacity 相関だけを検査し、`Owned(region)` payload の `region_size` が `cap * size_of<T>` と一致することを確認していなかった。
- `push` / `replace` / sort / transform は `cap` と `len` を raw offset 計算に使うため、backing storage extent まで invariant に含めないと、capacity metadata が storage owner の実 byte 数を証明していない状態が残る。

## 問題

VecCopyInvariant validates len/initialized_len/cap/storage tag correlation but does not verify that VecStorage::Owned(region) has region_size == cap * size_of<T>. Raw push/sort/transform boundaries can therefore rely on cap metadata without a typed proof that the backing storage extent matches it.

## 影響

Memory safety proof remains incomplete at the current Copy-only Vec boundary: an internally malformed owner-backed aggregate can pass the invariant and reach raw store/load paths with insufficient backing extent. This also weakens the future OwnedBuffer/InitializedCell design by leaving storage capacity as convention instead of proof.

## 修正方針

Extend VecCopyInvariantInvalid and vec_buffer_current_copy_invariant<T> so Owned storage proves the RegionToken extent against capacity and element size using checked arithmetic. Keep callers matching the typed invariant enum, and add regression policy/doctest coverage that rejects removing the extent proof.

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/issues.js check`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/invariant.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-push.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-root.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/get.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-get.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-sort.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/map.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-map.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-filter.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/aggregate.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-aggregate.json -j 1 --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-region-extent-data.json -j 1 --assert-io`

## 2026-05-20 Agent 1 修正

`VecCopyInvariantInvalid` に `OwnedStorageExtentMismatch` を追加し、`vec_buffer_current_copy_invariant<T>` の `VecStorage::Owned(region)` branch で `size_of<T>`、allocator payload 上限、`cap * size_of<T>`、`region_size(region)` を検査するようにした。

これにより current Copy-only `Vec` raw boundary は、metadata と storage tag だけでなく、backing `RegionToken` の byte extent も同じ typed invariant proof に含める。`cap` が allocator 上限を超える場合や、`region_size` と expected byte 数が一致しない場合は `Invalid(OwnedStorageExtentMismatch)` として raw access へ進まない。

回帰検査として `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に extent proof を追加し、`VecCopyInvariant` が bool や tag-only proof へ戻らないようにした。
