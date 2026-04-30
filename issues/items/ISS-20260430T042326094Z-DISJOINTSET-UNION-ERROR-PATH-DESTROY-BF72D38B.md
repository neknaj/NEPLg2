---
id: ISS-20260430T042326094Z-DISJOINTSET-UNION-ERROR-PATH-DESTROY-BF72D38B
title: "DisjointSet union error path destroys owner instead of returning it"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/disjoint_set.nepl, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, nodesrc/test_stdlib_disjoint_set_union_error_owner.js"
---

# ISS-20260430T042326094Z-DISJOINTSET-UNION-ERROR-PATH-DESTROY-BF72D38B: DisjointSet union error path destroys owner instead of returning it

## 概要

DisjointSet.union takes ownership of DisjointSet, but its invalid-index branch frees the input set and returns Result::Err Diag. A recoverable union error should not silently destroy the collection owner.

## 対象

- `stdlib/alloc/collections/disjoint_set.nepl, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, nodesrc/test_stdlib_disjoint_set_union_error_owner.js`

## 根拠

- `DisjointSet.union` は `DisjointSet` を値で受け取り、成功時には更新後 owner を返す update API だった。
- 範囲外 branch は `free dsu` してから `Err(Diag)` を返しており、caller は失敗した union 後に元の set を回収できなかった。
- SegmentTree / Fenwick / BitSet / AdjacencyMatrix は owner 返却付き update error contract に移行済みであり、DisjointSet だけが internal free で recoverability を失っていた。

## 問題

DisjointSet.union takes ownership of DisjointSet, but its invalid-index branch frees the input set and returns Result::Err Diag. A recoverable union error should not silently destroy the collection owner.

## 影響

Callers cannot inspect the diagnostic and then decide how to reuse or free the original set. This keeps DisjointSet inconsistent with the owner-returning update-error contracts used by other raw-memory-backed collections.

## 修正方針

Introduce DisjointSetUpdateError with owner and diag fields, change union to return Result<DisjointSet, DisjointSetUpdateError>, update doctests/tests to recover and free the owner on Err, and add source-policy regression coverage.

## 検証

Run DisjointSet doctests, stdlib/tests/disjoint_set.n.md, tests/stdlib/disjoint_set_collections.n.md, source-policy regressions, and issue checks.

確認済み:

- `node nodesrc/test_stdlib_disjoint_set_union_error_owner.js`: passed
- `node nodesrc/test_stdlib_disjoint_set_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl --no-tree -o tmp/disjoint-set-union-error-owner-doctests.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md --no-tree -o tmp/disjoint-set-union-error-owner-stdlib-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/disjoint_set_collections.n.md --no-tree -o tmp/disjoint-set-union-error-owner-collections-tests.json -j 1` (`total=4`, `passed=4`, `failed=0`)

## 修正内容

- `DisjointSetUpdateError` を追加し、`owner <DisjointSet>` と `diag <Diag>` を分けた。
- `disjoint_set_update_error_diag` / `disjoint_set_update_error_owner` を追加し、診断の借用観察と owner 回収を API として分けた。
- `union` の戻り値を `Result<DisjointSet, DisjointSetUpdateError>` に変更し、範囲外 branch では元 owner を `DisjointSetUpdateError` に入れて返すようにした。
- stdlib / collection tests に、Err 後に owner を回収して `free` する回帰テストを追加した。
- `nodesrc/test_stdlib_disjoint_set_union_error_owner.js` を source policy に登録し、`Err(Diag)` や内部 `free` に戻る再発を検出するようにした。
