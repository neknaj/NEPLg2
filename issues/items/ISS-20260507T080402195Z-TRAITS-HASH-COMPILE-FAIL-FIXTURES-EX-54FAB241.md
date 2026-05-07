---
id: ISS-20260507T080402195Z-TRAITS-HASH-COMPILE-FAIL-FIXTURES-EX-54FAB241
title: "traits_hash compile_fail fixtures expect stale move diagnostic id"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: tests/stdlib/traits_hash.n.md
---

# ISS-20260507T080402195Z-TRAITS-HASH-COMPILE-FAIL-FIXTURES-EX-54FAB241: traits_hash compile_fail fixtures expect stale move diagnostic id

## 概要

The current compiler reports resource.cell.moved for non-Copy generic values consumed twice, but traits_hash compile_fail fixtures still expect the stale resource.move.use_moved id. The doctests fail with diagnostic code mismatch even though the Resource IR check is rejecting the program correctly.

## 対象

- `tests/stdlib/traits_hash.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/traits-hash-current.json -j 1 --dist web/dist`: total=6, passed=4, failed=2。
- 失敗した 2 件はいずれも `diag_code: resource.move.use_moved` を期待していたが、実際の compiler 出力は `resource.cell.moved` と `resource.cell.uninit` だった。
- `doc/neplg2/compiler_diagnostics_redesign_plan.md` では Resource IR cell state diagnostic として `resource.cell.moved` が整理済みであり、fixture の目的は non-Copy value の二重消費検出なので `resource.cell.moved` を期待するのが現在の設計に合う。

## 問題

The current compiler reports resource.cell.moved for non-Copy generic values consumed twice, but traits_hash compile_fail fixtures still expect the stale resource.move.use_moved id. The doctests fail with diagnostic code mismatch even though the Resource IR check is rejecting the program correctly.

## 影響

The hash trait regression suite no longer verifies HashKey and Hasher Copy-bound behavior on current main, and broad stdlib hash validation is blocked by stale diagnostic metadata rather than a semantic failure.

## 修正方針

Update the affected compile_fail fixtures to expect resource.cell.moved, preserving the programs and their static-check purpose. Confirm focused traits_hash and hash collection suites pass.

## 検証

- `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/traits-hash-resource-cell-diag.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md -i tests/stdlib/hash_collection_rehash.n.md -i tests/stdlib/traits_hash.n.md -i tests/stdlib/collections_diag.n.md -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/traits-hash-resource-cell-hash-suite.json -j 1 --dist web/dist`: total=25, passed=25

## 対応結果

`tests/stdlib/traits_hash.n.md` の `hashkey_bound_is_not_copy_bound` と `hasher_bound_is_not_copy_bound` の `diag_code` を `resource.cell.moved` に更新した。fixture 本体は変更せず、HashKey / Hasher が Copy bound を暗黙に与えないことを現在の Resource IR diagnostic ID で固定した。
