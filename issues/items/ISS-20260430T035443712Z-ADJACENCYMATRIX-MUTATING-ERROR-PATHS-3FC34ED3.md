---
id: ISS-20260430T035443712Z-ADJACENCYMATRIX-MUTATING-ERROR-PATHS-3FC34ED3
title: "AdjacencyMatrix mutating error paths consume owner without cleanup or return"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/adjacency_matrix.nepl, stdlib/tests/adjacency_matrix.n.md, tests/stdlib/adjacency_matrix_collections.n.md, nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js"
---

# ISS-20260430T035443712Z-ADJACENCYMATRIX-MUTATING-ERROR-PATHS-3FC34ED3: AdjacencyMatrix mutating error paths consume owner without cleanup or return

## 概要

AdjacencyMatrix.insert and AdjacencyMatrix.remove take AdjacencyMatrix by value and return Result<AdjacencyMatrix, Diag>, but their out-of-bounds branches return Err(Diag) without returning the input owner or freeing matrix storage.

## 対象

- `stdlib/alloc/collections/adjacency_matrix.nepl, stdlib/tests/adjacency_matrix.n.md, tests/stdlib/adjacency_matrix_collections.n.md`

## 根拠

- `stdlib/alloc/collections/adjacency_matrix.nepl` の `insert` / `remove` は `AdjacencyMatrix` owner を値で受け取り、`Result<AdjacencyMatrix, Diag>` を返していた。
- 範囲外 branch は `diag_err<AdjacencyMatrix> adjacency_matrix_vertex_diag` を返すだけで、入力 `g` の `bits` owner を `free` せず、`Err` payload にも戻していなかった。
- `contains` / `len` は既に `&AdjacencyMatrix` receiver へ修正済みだったため、残っていた問題は mutating API の失敗時 owner contract だった。

## 問題

AdjacencyMatrix.insert and AdjacencyMatrix.remove take AdjacencyMatrix by value and return Result<AdjacencyMatrix, Diag>, but their out-of-bounds branches return Err(Diag) without returning the input owner or freeing matrix storage.

## 影響

Invalid graph updates leave callers without an ownership-safe way to recover or dispose of matrix storage. This is the same mutating error-path owner contract problem found in BitSet and Fenwick.

## 修正方針

Introduce an owner-carrying AdjacencyMatrixUpdateError or equivalent mutating error contract, make invalid updates return the original owner with the diagnostic, and update tests to recover and free the owner on Err.

## 検証

Add focused doctests/source-policy regressions that trigger invalid insert/remove and then recover and free the returned AdjacencyMatrix owner with Resource owner checking enabled.

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl --no-tree -o tmp/adjacency-update-error-owner-doctests-after-pull.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/adjacency_matrix.n.md --no-tree -o tmp/adjacency-update-error-owner-stdlib-tests-after-pull.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/adjacency_matrix_collections.n.md --no-tree -o tmp/adjacency-update-error-owner-collections-tests-after-pull.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `AdjacencyMatrixUpdateError` を追加し、`owner <AdjacencyMatrix>` と `diag <Diag>` を分けて `insert` / `remove` の失敗時に元の owner と診断を返す contract にした。
- `insert` / `remove` の戻り値を `Result<AdjacencyMatrix, AdjacencyMatrixUpdateError>` に変更し、範囲外 branch で `AdjacencyMatrixUpdateError g d` を返すようにした。
- `adjacency_matrix_update_error_diag` / `adjacency_matrix_update_error_owner` を追加し、診断の借用観察と owner 回収を API として分離した。
- `stdlib/tests/adjacency_matrix.n.md` と `tests/stdlib/adjacency_matrix_collections.n.md` に、Err 後に owner を回収して `free` する回帰テストを追加した。
- `nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js` を source policy に登録し、`Err(Diag)` へ戻って owner を失う再発を検出するようにした。
