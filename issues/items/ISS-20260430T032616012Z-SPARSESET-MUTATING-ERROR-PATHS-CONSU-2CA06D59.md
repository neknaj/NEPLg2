---
id: ISS-20260430T032616012Z-SPARSESET-MUTATING-ERROR-PATHS-CONSU-2CA06D59
title: "SparseSet mutating error paths consume owner without cleanup or return"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/sparse_set.nepl, stdlib/tests/sparse_set.n.md"
---

# ISS-20260430T032616012Z-SPARSESET-MUTATING-ERROR-PATHS-CONSU-2CA06D59: SparseSet mutating error paths consume owner without cleanup or return

## 概要

SparseSet.insert and SparseSet.remove take SparseSet by value and return Result<SparseSet, Diag>, but their out-of-bounds Err branches return only Diag. The input owner is neither returned nor freed, so callers that hit invalid update paths cannot recover or dispose of dense/sparse storage safely.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl, stdlib/tests/sparse_set.n.md`

## 根拠

- `stdlib/alloc/collections/sparse_set.nepl` の旧 `insert` / `remove` は `SparseSet` を値で受け取る一方、範囲外 branch で `Err(Diag)` だけを返していた。
- 旧 raw header layout では dense/sparse owner が header 内の raw address へ隠れており、caller は invalid update 後に owner を再利用も cleanup もできなかった。
- borrowed observer の回帰テストで `free` を明示したところ、raw header から読み戻した pointer を ResourceIR が owner として扱えず、storage representation の修正が必要だと確認した。

## 問題

SparseSet.insert and SparseSet.remove take SparseSet by value and return Result<SparseSet, Diag>, but their out-of-bounds Err branches return only Diag. The input owner is neither returned nor freed, so callers that hit invalid update paths cannot recover or dispose of dense/sparse storage safely.

## 影響

Invalid SparseSet updates create an API contract that cannot be used safely under strict owner checking. Tests can accidentally hide this by only checking is_err after moving the owner.

## 修正方針

Redesign SparseSet mutating error semantics so invalid updates either borrow the owner and leave it usable, or return an error payload that carries the original SparseSet owner for cleanup. Update tests to prove owner recovery or cleanup on Err.

## 検証

Add tests that trigger invalid insert/remove and then either reuse and free the borrowed owner or free the owner returned in the error payload, with Resource owner checking enabled.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md --no-tree -o tmp/sparse-set-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/sparse-set-collections-borrowed-observers.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-doctest-borrowed-observers.json -j 1` (`total=7`, `passed=7`, `failed=0`)
- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_sparse_set_borrowed_observers.js`: passed

## 修正内容

- `SparseSet` を `hdr <i32>` raw header から `n/len0/dense/sparse` typed fields へ移行し、dense/sparse owner を構造体フィールドとして ResourceIR が追跡できる形にした。
- `sparse_set_free_arrays` を追加し、`free` と invalid update cleanup が同じ dense/sparse owner cleanup boundary を使うようにした。
- `insert` / `remove` は consumed owner の fields を値で取り出し、成功時は新しい `SparseSet` に owner を移し、範囲外 `Err` では dense/sparse storage を解放してから `Err(Diag)` を返すようにした。
- invalid index test に `remove` の Err path を追加し、`contains` の borrowed Err path では owner を `free` する形に更新した。
