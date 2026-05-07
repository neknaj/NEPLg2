---
id: ISS-20260507T105015762Z-BTREE-ARRAY-COST-FIXTURES-STILL-CALL-51BA8332
title: "btree array cost fixtures still call borrowed observers by value"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "tests/stdlib/btree_array_cost.n.md, nodesrc/test_stdlib_btree_borrowed_observers.js"
---

# ISS-20260507T105015762Z-BTREE-ARRAY-COST-FIXTURES-STILL-CALL-51BA8332: btree array cost fixtures still call borrowed observers by value

## 概要

After BTreeMap/BTreeSet observer APIs became borrowed primary APIs, tests/stdlib/btree_array_cost.n.md still calls sorted_array_map_len/get and sorted_array_set_len/contains by value. Focused doctests now fail with type.overload.type_args_mismatch.

## 対象

- `tests/stdlib/btree_array_cost.n.md, nodesrc/test_stdlib_btree_borrowed_observers.js`

## 根拠

- `tests/stdlib/btree_array_cost.n.md` は `sorted_array_map_len` / `sorted_array_map_get` / `sorted_array_set_len` / `sorted_array_set_contains` を owner by-value で呼び出していた。
- remote main の BTree observer 修正により、これらの alias は primary borrowed observer を指すため、fixture は `type.overload.type_args_mismatch` で全 doctest が compile fail していた。
- `nodesrc/test_stdlib_btree_borrowed_observers.js` は stdlib BTree tests と pipe collection tests だけを見ており、cost fixture の alias 呼び出し退行を検出できていなかった。

## 問題

After BTreeMap/BTreeSet observer APIs became borrowed primary APIs, tests/stdlib/btree_array_cost.n.md still calls sorted_array_map_len/get and sorted_array_set_len/contains by value. Focused doctests now fail with type.overload.type_args_mismatch.

## 影響

The cost fixture no longer runs, so compile-time/runtime cost tracking for sorted-array BTree aliases cannot detect regressions after the owner-flow API change.

## 修正方針

Update the cost fixture to borrow the map/set for read-only sorted-array observer aliases, explicitly free the owner afterwards, and extend the BTree borrowed-observer source policy to cover this fixture.

## 検証

node nodesrc/test_stdlib_btree_borrowed_observers.js; node nodesrc/tests.js -i tests/stdlib/btree_array_cost.n.md --no-tree -o tmp/btree-array-cost-borrowed-observers.json -j 1 --dist web/dist

## 2026-05-07 対応結果

- `tests/stdlib/btree_array_cost.n.md` の sorted-array observer alias 呼び出しを `&m` / `&s` の borrowed receiver に変更した。
- 観測後に `sorted_array_map_free` / `sorted_array_set_free` を呼ぶ形にし、cost fixture でも owner 解放を明示した。
- `nodesrc/test_stdlib_btree_borrowed_observers.js` に cost fixture の by-value sorted-array observer alias 再導入を拒否する regression を追加した。
- focused verification:
  - `node nodesrc/test_stdlib_btree_borrowed_observers.js`
  - `node nodesrc/tests.js -i tests/stdlib/btree_array_cost.n.md --no-tree -o tmp/btree-array-cost-borrowed-observers.json -j 1 --dist web/dist`
