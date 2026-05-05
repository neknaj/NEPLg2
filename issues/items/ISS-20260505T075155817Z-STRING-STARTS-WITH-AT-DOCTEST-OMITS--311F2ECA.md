---
id: ISS-20260505T075155817Z-STRING-STARTS-WITH-AT-DOCTEST-OMITS--311F2ECA
title: "string starts-with-at doctest omits stdout assertion report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/alloc/string.nepl
---

# ISS-20260505T075155817Z-STRING-STARTS-WITH-AT-DOCTEST-OMITS--311F2ECA: string starts-with-at doctest omits stdout assertion report

## 概要

The str_starts_with_at doc-comment doctest builds six std/test assertions but returns checks_exit_code checks without printing a deterministic assertion report.

## 対象

- `stdlib/alloc/string.nepl`

## 根拠

- `stdlib/alloc/string.nepl::doctest#3` は `str_starts_with_at` の true/false boundary を `std/test` checks 6 件で確認していた。
- 修正前は `checks_exit_code checks` だけを返し、stdout report は空だった。
- 変更前の focused run は pass していたため、今回は観測契約の不足だけを修正対象にした。

## 問題

The str_starts_with_at doc-comment doctest builds six std/test assertions but returns checks_exit_code checks without printing a deterministic assertion report.

## 影響

Prefix matching regressions are visible only through exit success, and runner parity does not verify the assertion report for this parser-facing string helper.

## 修正方針

Add exit_code metadata and a checks_print_report stdout fixture for the str_starts_with_at doctest without changing the byte-oriented semantics.

## 対応結果

- doctest metadata に `exit_code: 0` と `Checked [ok,ok,ok,ok,ok,ok]` stdout fixture を追加した。
- `checks_print_report` の戻り値を `checks_exit_code` に渡し、byte-oriented prefix matching の実装は変更しなかった。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 3 --dist web/dist`: passed, stdout=`Checked [ok,ok,ok,ok,ok,ok]`
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
