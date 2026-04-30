---
id: ISS-20260430T034929316Z-BITSET-MUTATING-ERROR-PATHS-CONSUME--2C953144
title: "BitSet mutating error paths consume owner without cleanup or return"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/bitset.nepl, stdlib/tests/bitset.n.md, tests/stdlib/bitset_collections.n.md, nodesrc/test_stdlib_bitset_update_error_owner.js"
---

# ISS-20260430T034929316Z-BITSET-MUTATING-ERROR-PATHS-CONSUME--2C953144: BitSet mutating error paths consume owner without cleanup or return

## 概要

BitSet.insert and BitSet.remove take BitSet by value and return Result<BitSet, Diag>, but their out-of-bounds branches return Err(Diag) without returning the input BitSet owner or freeing its internal bit storage.

## 対象

- `stdlib/alloc/collections/bitset.nepl, stdlib/tests/bitset.n.md, tests/stdlib/bitset_collections.n.md`

## 根拠

- `stdlib/alloc/collections/bitset.nepl` の `insert` / `remove` は `fn insert <(BitSet,i32)*>Result<BitSet, Diag>>` / `fn remove <(BitSet,i32)*>Result<BitSet, Diag>>` で `BitSet` owner を値で受け取っていた。
- 範囲外 branch は `diag_err<BitSet> bitset_index_diag` を返すだけで、入力 `bs` の `bits` owner を `free` せず、`Err` payload にも戻していなかった。
- `contains` / `len` は既に `&BitSet` receiver へ修正済みだったため、残っていた問題は mutating API の失敗時 owner contract だった。

## 問題

BitSet.insert and BitSet.remove take BitSet by value and return Result<BitSet, Diag>, but their out-of-bounds branches return Err(Diag) without returning the input BitSet owner or freeing its internal bit storage.

## 影響

Callers that hit an invalid BitSet update have no ownership-safe way to recover or dispose of the bit storage. This undermines mandatory memory-safety checking in the same owner-contract family as the Fenwick add error-path fix.

## 修正方針

Introduce an owner-carrying BitSetUpdateError or equivalent mutating error contract, make invalid updates return the original BitSet owner with the diagnostic, and update tests to recover and free the owner on Err.

## 検証

Add focused doctests/source-policy regressions that trigger invalid insert/remove and then recover and free the returned BitSet owner with Resource owner checking enabled.

確認済み:

- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl --no-tree -o tmp/bitset-update-error-owner-doctests.json -j 1` (`total=7`, `passed=7`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/bitset.n.md --no-tree -o tmp/bitset-update-error-owner-stdlib-tests.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/bitset-update-error-owner-collections-tests.json -j 1` (`total=3`, `passed=3`, `failed=0`)
- `node nodesrc/test_stdlib_bitset_update_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `BitSetUpdateError` を追加し、`owner <BitSet>` と `diag <Diag>` を分けて `insert` / `remove` の失敗時に元の owner と診断を返す contract にした。
- `insert` / `remove` の戻り値を `Result<BitSet, BitSetUpdateError>` に変更し、範囲外 branch で `BitSetUpdateError bs d` を返すようにした。
- `bitset_update_error_diag` / `bitset_update_error_owner` を追加し、診断の借用観察と owner 回収を API として分離した。
- `stdlib/tests/bitset.n.md` と `tests/stdlib/bitset_collections.n.md` に、Err 後に owner を回収して `free` する回帰テストを追加した。
- `nodesrc/test_stdlib_bitset_update_error_owner.js` を source policy に登録し、`Err(Diag)` へ戻って owner を失う再発を検出するようにした。
