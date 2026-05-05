---
id: ISS-20260505T075515489Z-ALLOC-STRING-SEARCH-AND-BOOL-DOCTEST-29C0BAB3
title: "alloc string search and bool doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/alloc/string.nepl
---

# ISS-20260505T075515489Z-ALLOC-STRING-SEARCH-AND-BOOL-DOCTEST-29C0BAB3: alloc string search and bool doctests omit stdout assertion reports

## 概要

The str_find, to_bool, and find doc-comment doctests build std/test reports but return checks_exit_code without printing deterministic assertion reports.

## 対象

- `stdlib/alloc/string.nepl`

## 根拠

- `stdlib/alloc/string.nepl::doctest#5` は `str_find` の empty / found / not-found / too-long pattern を4件の `std/test` check で検査していたが、stdout report を出していなかった。
- `stdlib/alloc/string.nepl::doctest#7` は `to_bool` の `"true"` / `"false"` 解析を2件の assertion で検査していたが、stdout report を出していなかった。
- `stdlib/alloc/string.nepl::doctest#10` は `find` の `Some` / `None` 境界を6件の assertion で検査していたが、stdout report を出していなかった。

## 問題

The str_find, to_bool, and find doc-comment doctests build std/test reports but return checks_exit_code without printing deterministic assertion reports.

## 影響

String search and bool parsing regressions are only observed through exit success, and runner parity does not verify their assertion report output.

## 修正方針

Add exit_code metadata and checks_print_report stdout fixtures to the three focused doctests without changing string search or parsing semantics.

## 対応結果

- `str_find` doctest に `exit_code: 0` と4件分の `Checked [ok,...]` stdout fixture を追加した。
- `to_bool` doctest に `exit_code: 0` と2件分の stdout fixture を追加した。
- `find` doctest に `exit_code: 0` と6件分の stdout fixture を追加した。
- 各 doctest は `checks_print_report` の結果を `checks_exit_code` に渡す形へ統一した。
- string search / bool parsing の実装本体は変更していない。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 5 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 7 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 10 --dist web/dist`: passed
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
- `node nodesrc/issues.js check`: passed
