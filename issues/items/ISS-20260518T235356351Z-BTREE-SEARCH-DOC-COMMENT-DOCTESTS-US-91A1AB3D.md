---
id: ISS-20260518T235356351Z-BTREE-SEARCH-DOC-COMMENT-DOCTESTS-US-91A1AB3D
title: "BTree search doc-comment doctests use legacy checks reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_btree_search_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T235356351Z-BTREE-SEARCH-DOC-COMMENT-DOCTESTS-US-91A1AB3D: BTree search doc-comment doctests use legacy checks reports without stdout fixtures

## 概要

BTreeMap and BTreeSet key equality doc-comment doctests still use checks_* reports and do not pin stdout / exit_code metadata, so their Copy-only equality boundary examples can pass without fixture-checked assertion details.

## 対象

- `stdlib/alloc/collections/btreemap/search.nepl, stdlib/alloc/collections/btreeset/search.nepl, nodesrc/test_stdlib_btree_search_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `btreemap_key_eq` / `btreeset_key_eq` は、by-value `Ord` 比較を 2 回行う Copy-only equality boundary を説明する doc-comment doctest を持っていた。
- しかし両 doctest は `checks_print_report` を呼ぶだけで、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。
- さらに旧 `checks_*` helper のままだったため、現行方針の named `TestReport` による assertion label / expected / actual fixture と揃っていなかった。

## 問題

BTreeMap and BTreeSet key equality doc-comment doctests still use checks_* reports and do not pin stdout / exit_code metadata, so their Copy-only equality boundary examples can pass without fixture-checked assertion details.

## 影響

The doctests document a memory-safety boundary for by-value Ord comparisons, but runner compatibility and expected/actual details are not fixed as stdout fixtures.

## 修正方針

Migrate both doc-comment doctests to named TestReport stdout fixtures, add exit_code metadata, and add a source policy contract that rejects checks_* / ret-only regression.

## 検証

Run the focused BTree search doc-comment doctests, the source policy contract, issue checks, and diff whitespace check.

## 対応結果

- `btreemap_key_eq` の doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- `btreeset_key_eq` の doc-comment doctest も同じく stdout report + exit_code fixture へ移行した。
- どちらも旧 `checks_*` から named `TestReport` API へ移行し、equal / unequal の2観測を assertion label と expected / actual 付きで固定した。
- `nodesrc/test_stdlib_btree_search_doc_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式への退行を source policy で拒否する。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
