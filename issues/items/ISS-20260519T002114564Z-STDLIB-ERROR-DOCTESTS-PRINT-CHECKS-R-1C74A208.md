---
id: ISS-20260519T002114564Z-STDLIB-ERROR-DOCTESTS-PRINT-CHECKS-R-1C74A208
title: "stdlib error doctests print checks reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/error.n.md, nodesrc/test_stdlib_error_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T002114564Z-STDLIB-ERROR-DOCTESTS-PRINT-CHECKS-R-1C74A208: stdlib error doctests print checks reports without stdout fixtures

## 概要

stdlib/tests/error.n.md has three diagnostic value-model doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and still use the legacy checks_* report path.

## 対象

- `stdlib/tests/error.n.md, nodesrc/test_stdlib_error_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/error.n.md` の3件は `StdErrorKind` / `Diag` / `Diags` / `Outcome` / result-like helper の値モデルを検査していた。
- 旧実装は `checks_print_report` / `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなく、report の assertion label や expected / actual が fixture として固定されていなかった。
- 診断値モデルは静的検査・selfhost 側のエラー処理にも関わるため、単なる終了コードではなく構造化 report の内容を固定する必要がある。

## 問題

stdlib/tests/error.n.md has three diagnostic value-model doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and still use the legacy checks_* report path.

## 影響

StdErrorKind, Diag, Diags, Outcome, and result-like helper behavior can regress without fixture-checked assertion labels, expected values, and canonical stdout format.

## 修正方針

Migrate all three doctests to named TestReport stdout fixtures with exit_code metadata, and add a source policy contract that rejects ret-only or legacy checks_* regression.

## 検証

Run the error source policy contract, focused error doctest, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `std_error_kind_and_diag_value_model` を named `TestReport` へ移行し、kind string、Diag message、Diag kind、span file id、source、Diags length、error 有無を stdout fixture として固定した。
- `outcome_helpers_keep_result_and_diags_separate` を named `TestReport` へ移行し、Ok/Err の result と Diags の分離、warning-only の非 error 判定、`result_to_outcome` の Err kind を fixture として固定した。
- `result_and_outcome_common_helpers` を named `TestReport` へ移行し、Result / Outcome 共通 helper の ok/err 判定、result 取り出し、Diags 継承を stdout fixture として固定した。
- `nodesrc/test_stdlib_error_nmd_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式、enum kind 分岐の退行を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
