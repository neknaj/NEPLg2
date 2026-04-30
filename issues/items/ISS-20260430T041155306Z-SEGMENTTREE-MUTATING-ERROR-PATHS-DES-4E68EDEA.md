---
id: ISS-20260430T041155306Z-SEGMENTTREE-MUTATING-ERROR-PATHS-DES-4E68EDEA
title: "SegmentTree mutating error paths destroy owner instead of returning it"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, nodesrc/test_stdlib_segment_tree_update_error_owner.js"
---

# ISS-20260430T041155306Z-SEGMENTTREE-MUTATING-ERROR-PATHS-DES-4E68EDEA: SegmentTree mutating error paths destroy owner instead of returning it

## 概要

SegmentTree.replace and SegmentTree.add take ownership of SegmentTree, but their invalid-index branches free the input tree and return Result::Err Diag. A recoverable update error should not silently destroy the collection owner.

## 対象

- `stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, nodesrc/test_stdlib_segment_tree_update_error_owner.js`

## 根拠

- `stdlib/alloc/collections/segment_tree.nepl` の `replace` / `add` は `SegmentTree` を値で受け取り、成功時に更新後 owner を返す API だった。
- 一方で範囲外 branch は `free st` してから `Err(Diag)` を返しており、caller は失敗した update の後で元の tree を回収できなかった。
- Fenwick / BitSet / AdjacencyMatrix は、同じ update API で owner 返却付き error 型を使う contract に移行済みだった。
- SegmentTree だけが「recoverable な Result error が owner を破棄する」設計として残っていた。

## 問題

SegmentTree.replace and SegmentTree.add take ownership of SegmentTree, but their invalid-index branches free the input tree and return Result::Err Diag. A recoverable update error should not silently destroy the collection owner.

## 影響

Callers cannot inspect the diagnostic and then decide how to reuse or free the original tree. This keeps SegmentTree inconsistent with the owner-returning update-error contracts already used by Fenwick, BitSet, and AdjacencyMatrix.

## 修正方針

Introduce SegmentTreeUpdateError with owner and diag fields, change replace/add to return Result<SegmentTree, SegmentTreeUpdateError>, update doctests/tests to recover and free the owner on Err, and add source-policy regression coverage.

## 検証

Run SegmentTree doctests, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md, source-policy regressions, and issue checks.

確認済み:

- `node nodesrc/test_stdlib_segment_tree_update_error_owner.js`: passed
- `node nodesrc/test_stdlib_segment_tree_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/segment-tree-update-error-owner-doctests.json -j 1` (`total=5`, `passed=5`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segment-tree-update-error-owner-stdlib-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/segment-tree-update-error-owner-collections-tests.json -j 1` (`total=3`, `passed=3`, `failed=0`)

## 修正内容

- `SegmentTreeUpdateError` を追加し、失敗時の `owner <SegmentTree>` と `diag <Diag>` を分離した。
- `segment_tree_update_error_diag` / `segment_tree_update_error_owner` を追加し、診断の借用観察と owner 回収を API として分けた。
- `replace` / `add` の戻り値を `Result<SegmentTree, SegmentTreeUpdateError>` に変更し、範囲外 branch では元 owner を `SegmentTreeUpdateError` に入れて返すようにした。
- stdlib / collection tests に、Err 後に owner を回収して `free` する回帰テストを追加した。
- `nodesrc/test_stdlib_segment_tree_update_error_owner.js` を source policy に登録し、`Err(Diag)` や内部 `free` に戻る再発を検出するようにした。
