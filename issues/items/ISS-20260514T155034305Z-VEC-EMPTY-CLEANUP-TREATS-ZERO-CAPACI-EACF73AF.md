---
id: ISS-20260514T155034305Z-VEC-EMPTY-CLEANUP-TREATS-ZERO-CAPACI-EACF73AF
title: "Vec empty cleanup treats zero-capacity sentinel as owned storage"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-16
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

## 2026-05-16 Agent 1 regression 再修正

`RegionToken<T>` の direct raw owner field 化と public `alloc_ptr` API 撤去後の main で再確認したところ、`vec_free_storage<T>` が再び `storage` 引数を無視し、`VecStorageState::Empty` の zero-size sentinel を常時 `dealloc_region` へ渡す形に戻っていた。

この調査で、上記の「`Empty` は no-op、`Owned` だけ `dealloc_region`」という修正方針は現行型では静的に正しくないことが分かった。`VecStorageState` と `RegionToken<T>` が独立した field / 引数であるため、`vec_free_storage<T>(VecStorageState::Empty, owned_region)` が型上成立する。この場合に Empty branch が no-op だと Resource IR が `RegionToken.raw` owner leak を報告するのは正しい。

そのため、この issue は再オープンする。短期的には `vec_free_storage<T>` を unconditional owner-consuming destructor に戻し、free obligation leak を避ける。`Empty` sentinel の runtime allocation は `dealloc_raw` の `ptr <= 0` no-op で返されない。

根本修正は `ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134` として分離した。`VecStorage<T>::Empty | VecStorage<T>::Owned(RegionToken<T>)` のように owner token を Owned variant に構造的に束ね、borrowed observer / mutation もその型で表せるようにするまでは、source policy が Empty no-op を要求してはいけない。

## 2026-05-16 Agent 1 最終解決

`ISS-20260515T223330574Z-VEC-STORAGE-TAG-AND-REGIONTOKEN-OWNE-DDDAD134` の解決により、当初の blocker だった split `VecStorageState` / `RegionToken<T>` field は消えた。現在の `Vec<T>` は `len/cap/storage` だけを持ち、`VecStorage<T>::Empty | VecStorage<T>::Owned(RegionToken<T>)` が storage state と free obligation owner を同じ enum で表す。

`vec_free_storage<T>` は `(VecStorage<T>) -> ()` を受け取り、`Empty` では no-op、`Owned region` では `dealloc_region<T> region` を呼ぶ。`Empty` と allocated `RegionToken<T>` を同時に渡す signature は存在しないため、empty cleanup の no-op は stdlib 固有の暗黙相関ではなく source type と `match` の網羅性で証明される。

この issue の「zero-capacity sentinel を owned storage として dealloc path に流す」問題は閉じた。残る non-Copy payload drop traversal / initialized prefix / `OwnedBuffer<T>` は、`RV-STDLIB-004` と `STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR` の Stage 6 残件として継続する。

追加検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/agent1-vec-storage-owner-state-vec-tests.json -j 1 --dist web/dist --assert-io`: 6/6 passed
- `node nodesrc/run_doctest.js -i tests/stdlib/collection_cleanup_contract.n.md -n 5 --dist web/dist`: passed
- `node nodesrc/issues.js check --dir issues`: passed
