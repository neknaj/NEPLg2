---
id: ISS-20260430T035924737Z-SEGMENTTREE-UPDATE-AND-OBSERVER-APIS-E03209E2
title: "SegmentTree update and observer APIs do not preserve owner contract under ResourceIR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md"
---

# ISS-20260430T035924737Z-SEGMENTTREE-UPDATE-AND-OBSERVER-APIS-E03209E2: SegmentTree update and observer APIs do not preserve owner contract under ResourceIR

## 概要

SegmentTree.len still consumes SegmentTree by value for a read-only query, and update/error tests expose leaks under strict ResourceIR because successful query examples do not free returned owners while invalid update paths use Result<SegmentTree, Diag> without an owner-carrying error contract. stdlib/tests/segment_tree.n.md currently fails ResourceIR owner checking.

## 対象

- `stdlib/alloc/collections/segment_tree.nepl, stdlib/tests/segment_tree.n.md, tests/stdlib/segment_tree_collections.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segment-tree-before-owner-contract.json -j 1` で `stdlib/tests/segment_tree.n.md::doctest#1/#2` が ResourceIR owner leak として失敗した。
- `stdlib/alloc/collections/segment_tree.nepl` に `fn len <(SegmentTree)->i32>` が残っており、read-only length query が owner を消費していた。
- `replace` / `add` は `Result<SegmentTree, Diag>` を返しており、失敗時に元の `SegmentTree` owner を型として返せなかった。
- tests/doctests は `sum_range &st` 後に `free st` しない箇所が残り、strict owner checking 下で tree storage leak を検出した。

## 問題

SegmentTree.len still consumes SegmentTree by value for a read-only query, and update/error tests expose leaks under strict ResourceIR because successful query examples do not free returned owners while invalid update paths use Result<SegmentTree, Diag> without an owner-carrying error contract. stdlib/tests/segment_tree.n.md currently fails ResourceIR owner checking.

## 影響

SegmentTree cannot be used safely as a self-host collection under mandatory memory safety. Callers either leak tree storage after read-only queries/tests or lose the owner contract on failed updates.

## 修正方針

Align SegmentTree with the recent BitSet/Fenwick owner contracts: make read-only length borrow, introduce an owner-carrying update error type for replace/add failures, update tests/doctests to recover/free owners and free successful trees, and add source policy for the contract.

## 検証

Run SegmentTree stdlib tests, collection tests, doctests, source policy, issues check, and diff check with ResourceIR owner checking enabled.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md --no-tree -o tmp/segment-tree-owner-contract-stdlib-after-60af681c.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/segment-tree-owner-contract-collections-after-60af681c.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/segment-tree-owner-contract-doctests-after-60af681c.json -j 1` (`total=5`, `passed=5`, `failed=0`)
- `node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_segment_tree_borrowed_observers.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed (`files=436`)
- `git diff --check`: passed

## 修正内容

- `SegmentTreeUpdateError` を追加し、`tree <SegmentTree>` と `diag <Diag>` を分けて `replace` / `add` の失敗時に元の owner と診断を返す contract にした。
- `update_error_diag` / `update_error_tree` を追加し、診断の借用観察と owner 回収を API として分離した。
- `replace` / `add` の戻り値を `Result<SegmentTree, SegmentTreeUpdateError>` に変更し、範囲外 branch で元の owner を `Err` に入れて返すようにした。
- `len` を `&SegmentTree` receiver に変更し、read-only length query が owner を消費しないようにした。
- doctest / stdlib tests / collection tests を、成功 path では query 後に `free` し、失敗 path では `update_error_tree` で owner を回収して `free` する形に更新した。
- `nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js` に borrowed `len` と owner-carrying update error contract の source policy を追加した。
