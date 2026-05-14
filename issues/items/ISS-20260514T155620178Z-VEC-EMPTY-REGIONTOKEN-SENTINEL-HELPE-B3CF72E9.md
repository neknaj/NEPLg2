---
id: ISS-20260514T155620178Z-VEC-EMPTY-REGIONTOKEN-SENTINEL-HELPE-B3CF72E9
title: "Vec empty RegionToken sentinel helper is publicly exported"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/storage/view.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T155620178Z-VEC-EMPTY-REGIONTOKEN-SENTINEL-HELPE-B3CF72E9: Vec empty RegionToken sentinel helper is publicly exported

## 概要

vec_empty_region<T> constructs the zero-size RegionToken sentinel used internally by VecStorageState::Empty, but it is declared pub and is re-exported through the Vec storage facade. External code can depend on a transitional RegionToken sentinel helper that should not be part of the public Vec API.

## 対象

- `stdlib/alloc/collections/vec/storage/view.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `vec_empty_region<T>` は `RegionToken<T>` を組み立てる helper であり、`VecStorageState::Empty` の内部 sentinel 以外に用途がない。
- `storage/view.nepl` は `storage.nepl` から `as @merge` で re-export されるため、`pub fn vec_empty_region` のままだと root `alloc/collections/vec` API からも sentinel helper を参照できる。

## 問題

vec_empty_region<T> constructs the zero-size RegionToken sentinel used internally by VecStorageState::Empty, but it is declared pub and is re-exported through the Vec storage facade. External code can depend on a transitional RegionToken sentinel helper that should not be part of the public Vec API.

## 影響

The Stage 6 owner-token boundary remains leakier than necessary: public callers can name the empty sentinel constructor instead of going through Vec empty constructors, making the future OwnedBuffer/storage-state replacement harder and normalizing RegionToken construction as API surface.

## 修正方針

Make vec_empty_region private to storage/view.nepl, keep pub vec_empty<T> as the typed empty Vec constructor, and add source policy coverage that vec_empty_region is not public while vec_empty remains public.

## 検証

Fixed.

- `vec_empty_region<T>` を private helper に変更した。
- public constructor は `vec_empty<T> -> Vec<T>` に限定し、呼び出し側が `RegionToken<T>` sentinel を直接作る API surface を閉じた。
- source policy に、`vec_empty<T>` は public のまま、`vec_empty_region<T>` は public に戻さない監視を追加した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/agent1-vec-empty-region-private.json -j 1 --dist web/dist --assert-io`: 3/3 passed
- `node nodesrc/issues.js check`: passed
