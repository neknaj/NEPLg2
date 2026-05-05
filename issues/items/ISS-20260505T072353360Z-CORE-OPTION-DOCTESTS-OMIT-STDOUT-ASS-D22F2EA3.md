---
id: ISS-20260505T072353360Z-CORE-OPTION-DOCTESTS-OMIT-STDOUT-ASS-D22F2EA3
title: "core option doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/option.nepl
---

# ISS-20260505T072353360Z-CORE-OPTION-DOCTESTS-OMIT-STDOUT-ASS-D22F2EA3: core option doctests omit stdout assertion reports

## 概要

The core Option doc-comment doctests use std/test checks but rely on ret: 0 and checks_exit_code checks, so assertion details are not printed or fixed as stdout fixtures.

## 対象

- `stdlib/core/option.nepl`

## 根拠

- `stdlib/core/option.nepl` の doc-comment doctest 3 件は、すべて `std/test` checks を使っていた。
- 修正前は 3 件とも `ret: 0` metadata と `checks_exit_code checks` だけで成功を表し、stdout report を fixture として固定していなかった。
- 同ファイルの注意書きも `ret:` 比較を前提にしており、現在の `.n.md` / doc-comment doctest 方針とずれていた。

## 問題

The core Option doc-comment doctests use std/test checks but rely on ret: 0 and checks_exit_code checks, so assertion details are not printed or fixed as stdout fixtures.

## 影響

Option helper regressions lose deterministic assertion report coverage, and the docs still imply ret-based success for std target tests.

## 修正方針

Migrate the Option doctests to exit_code metadata, emit checks_print_report output, and update the surrounding documentation wording so std doctests are described as stdout report plus exit-code checks.

## 対応結果

- 3 件の doctest metadata を `ret: 0` から `exit_code: 0` + `stdout: mlstr:` に移行した。
- 各 doctest で `checks_print_report` を呼び、4 件 / 3 件 / 2 件の assertion report を fixture として固定した。
- 注意書きを `ret:` 比較ではなく stdout report と `exit_code:` で確認する説明へ更新した。

## 検証

- `node nodesrc/tests.js -i stdlib/core/option.nepl --no-tree -o tmp/core-option-report-agent1.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
