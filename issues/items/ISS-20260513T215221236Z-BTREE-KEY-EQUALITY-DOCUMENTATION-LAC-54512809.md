---
id: ISS-20260513T215221236Z-BTREE-KEY-EQUALITY-DOCUMENTATION-LAC-54512809
title: "BTree key equality documentation lacks doctests"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260513T215221236Z-BTREE-KEY-EQUALITY-DOCUMENTATION-LAC-54512809: BTree key equality documentation lacks doctests

## 概要

The Stage 6 Copy-only BTree key equality documentation added API comments without doctests, increasing the stdlib declaration doctest gap reported by the aggregate source policy.

## 対象

- `stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` で `nodesrc/test_stdlib_documentation_contract.js` が `stdlib declaration doctest gaps increased: 1033 > 1032` を報告した。
- 直前に追加した `btreemap_key_eq` / `btreeset_key_eq` のドキュメントコメントは、Copy-only equality boundary の説明を持つ一方で doctest を持っていなかった。

## 問題

The Stage 6 Copy-only BTree key equality documentation added API comments without doctests, increasing the stdlib declaration doctest gap reported by the aggregate source policy.

## 影響

The documentation policy requires every stdlib function/module/type to carry useful documentation and doctests; leaving the gap hides examples for the Copy-only equality boundary and makes the aggregate source policy warn.

## 修正方針

Add focused doctests that demonstrate true and false equality cases for BTreeMap and BTreeSet key equality helpers, then re-run the documentation contract and BTree focused tests.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js, focused BTree doctests, and issue checks.

## 対応結果

- `btreemap_key_eq<i32>` の true / false case を `std/test` report 付き doctest として追加した。
- `btreeset_key_eq<i32>` の true / false case を `std/test` report 付き doctest として追加した。
- Copy-only key equality boundary の説明と、利用例の doctest を同じ declaration に揃えた。

## 関連ドキュメント

- [stdlib documentation style guide](../../doc/neplg2/stdlib_documentation_style_guide.md)
- [stdlib documentation contract plan](../../doc/neplg2/stdlib_documentation_contract_plan.md)
