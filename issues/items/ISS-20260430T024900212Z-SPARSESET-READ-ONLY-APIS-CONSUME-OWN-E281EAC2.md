---
id: ISS-20260430T024900212Z-SPARSESET-READ-ONLY-APIS-CONSUME-OWN-E281EAC2
title: "SparseSet read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/sparse_set.nepl
---

# ISS-20260430T024900212Z-SPARSESET-READ-ONLY-APIS-CONSUME-OWN-E281EAC2: SparseSet read-only APIs consume owners by value instead of borrowing

## 概要

SparseSet len/universe_len/contains take SparseSet by value while reading only the header and arrays. Observing membership or length consumes the owner, so callers cannot safely free the dense/sparse storage afterwards.

## 対象

- `stdlib/alloc/collections/sparse_set.nepl`

## 根拠

- `stdlib/alloc/collections/sparse_set.nepl` に `fn len <(SparseSet)->i32>`、`fn universe_len <(SparseSet)->i32>`、`fn contains <(SparseSet,i32)*>Result<bool, Diag>>` が残っている。
- BitSet の owner-consuming observer 修正中に raw-array collection を確認し、header と dense/sparse arrays を読むだけの SparseSet observer も値 receiver のままだと判明した。

## 問題

SparseSet len/universe_len/contains take SparseSet by value while reading only the header and arrays. Observing membership or length consumes the owner, so callers cannot safely free the dense/sparse storage afterwards.

## 影響

SparseSet public observers encourage leaks or raw header workarounds in checked code. This conflicts with the no-technical-debt policy and with self-host collection use under strict owner checking.

## 修正方針

Redesign SparseSet observers around &SparseSet, update tests/doctests to borrow and then free, and remove by-value observer forms instead of retaining bad compatibility APIs.

## 検証

Add tests that run borrowed length and membership checks on one SparseSet, then free the same owner with owner checking enabled.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md --no-tree -o tmp/sparse-set-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/sparse-set-collections-borrowed-observers.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-doctest-borrowed-observers.json -j 1` (`total=7`, `passed=7`, `failed=0`)
- `node nodesrc/test_stdlib_sparse_set_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: passed

## 修正内容

- `SparseSet.len` / `SparseSet.universe_len` / `SparseSet.contains` を `&SparseSet` receiver に変更し、read-only API が owner を移動しない contract にした。
- 既存の raw header layout では `free` を追加した borrowed observer test が ResourceIR の owner 検査に通らなかったため、`SparseSet` 本体を `n/len0/dense/sparse` typed fields へ移行した。
- `stdlib/tests/sparse_set.n.md` と `tests/stdlib/sparse_set_collections.n.md` を、同一 owner に複数 observer を呼び、その後 `free` する回帰テストへ更新した。
- `nodesrc/test_stdlib_sparse_set_borrowed_observers.js` を追加し、by-value observer signature と by-value test usage が戻らないよう source policy に登録した。

## 関連して修正した issue

- `ISS-20260430T032616012Z-SPARSESET-MUTATING-ERROR-PATHS-CONSU-2CA06D59`: typed storage へ移行する過程で、`insert` / `remove` の invalid index Err path が consumed owner を cleanup してから `Err(Diag)` を返すようにした。
