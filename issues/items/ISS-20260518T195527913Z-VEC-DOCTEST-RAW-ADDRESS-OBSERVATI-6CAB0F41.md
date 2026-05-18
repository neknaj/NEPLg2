---
id: ISS-20260518T195527913Z-VEC-DOCTEST-RAW-ADDRESS-OBSERVATI-6CAB0F41
title: "Vec doctests observe raw data_mem_ptr address through internal helper"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/tests/vec.n.md, stdlib/alloc/collections/vec/access/data.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260518T195527913Z-VEC-DOCTEST-RAW-ADDRESS-OBSERVATI-6CAB0F41: Vec doctests observe raw data_mem_ptr address through internal helper

## 概要

`stdlib/tests/vec.n.md` と `Vec.data_mem_ptr` の使用例が、通常の `Vec` 動作確認のために `core/mem/internal` を import し、`mem_ptr_addr data_mem_ptr` で backing storage の raw address を観測していた。

## 対象

- `stdlib/tests/vec.n.md`
- `stdlib/alloc/collections/vec/access/data.nepl`
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- Stage 6 の方針では `MemPtr<T>` は non-owning pointer view であり、ordinary API / doctest は raw `i32` address を観測しない。
- `core/mem/internal` の `mem_ptr_addr` は raw address view boundary の実装補助であり、通常の `Vec` public behavior test が使うべき surface ではない。
- 既に compiler 側は ordinary source の raw helper use を source proof で制限しているため、stdlib の canonical test/documentation も raw address observer を「正常な使い方」として示してはいけない。

## 問題

`data_mem_ptr` は typed view observer として残しているが、その使用例と `stdlib/tests/vec.n.md` が raw address positivity を検証すると、`MemPtr` を raw storage identity として扱う旧設計を fixture が温存する。これにより、Stage 6 の `MemPtr = non-owning pointer` / `OwnedBuffer・VecStorage = free obligation owner` 分離がテストの意図から読み取りにくくなり、将来の raw boundary 強化時に fixture が不適切な権限を要求する。

## 影響

raw-memory-backed public API の整理が、compiler の source proof ではなく「この doctest は internal helper を import してよい」という慣習へ流れやすくなる。これは stdlib allowlist / module-specific proof を避け、source property から汎用的に証明する方針と衝突する。

## 修正方針

通常の `Vec` doctest は `with_capacity` / `is_empty` / `push` / `get` / `free` の public behavior で検証する。`data_mem_ptr` の使用例は typed `MemPtr<i32>` observer を得ても raw address へ落とさず、owner を保持して `free` できることを示す。source policy はこの fixture に `core/mem/internal` / `mem_ptr_addr` / raw pointer positivity check が戻らないことを監視する。

## 解決

- `stdlib/tests/vec.n.md` から `core/mem/internal` と `mem_ptr_addr data_mem_ptr` の raw address positivity check を削除し、`with_capacity` が空 `Vec` を返す public behavior assertion に置き換えた。
- `stdlib/alloc/collections/vec/access/data.nepl` の `data_mem_ptr` 使用例から `mem_ptr_addr` 観測を削除し、typed `MemPtr<i32>` observer と `Vec` owner cleanup の例に変更した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、通常 vec doctest / `data_mem_ptr` 使用例が `core/mem/internal` や `mem_ptr_addr` を使わないことを監視する policy を追加した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [raw-memory-backed APIs parent issue](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/agent1-vec-doctest-no-raw-address.json -j 1 --dist web/dist --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree -o tmp/agent1-vec-data-mem-ptr-doc-no-raw-address.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/issues.js check`: passed
