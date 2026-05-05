---
id: ISS-20260505T074155009Z-ALLOC-DIAG-OUTCOME-DOCTESTS-OMIT-STD-80083E0E
title: "alloc diag outcome doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/alloc/diag/error.nepl
---

# ISS-20260505T074155009Z-ALLOC-DIAG-OUTCOME-DOCTESTS-OMIT-STD-80083E0E: alloc diag outcome doctests omit stdout assertion reports

## 概要

The into_outcome and outcome_result doc-comment doctests build std/test checks but return checks_exit_code checks without emitting deterministic assertion reports.

## 対象

- `stdlib/alloc/diag/error.nepl`

## 根拠

- `stdlib/alloc/diag/error.nepl` の `into_outcome` / `outcome_result` doc-comment doctest は `std/test` checks を1件ずつ作っていた。
- 修正前は `checks_exit_code checks` だけを返し、Outcome helper の assertion report を stdout fixture として固定していなかった。

## 問題

The into_outcome and outcome_result doc-comment doctests build std/test checks but return checks_exit_code checks without emitting deterministic assertion reports.

## 影響

Outcome/diagnostic helper examples can regress their assertion report output while runner parity only observes exit success.

## 修正方針

Add exit_code metadata and checks_print_report stdout fixtures for the runnable Outcome doctests without changing diagnostic ownership behavior.

## 対応結果

- 対象 2 件に `exit_code: 0` と `Checked [ok]` stdout fixture を追加した。
- `checks_print_report` の戻り値を `checks_exit_code` に渡す形にし、診断 ownership / `diags_free` の挙動は変更しなかった。

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/diag/error.nepl --no-tree -o tmp/alloc-diag-error-report-agent1.json -j 1 --dist web/dist`: total=2, passed=2
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
