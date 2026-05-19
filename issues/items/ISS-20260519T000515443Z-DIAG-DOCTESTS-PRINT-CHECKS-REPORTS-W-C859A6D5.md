---
id: ISS-20260519T000515443Z-DIAG-DOCTESTS-PRINT-CHECKS-REPORTS-W-C859A6D5
title: "diag doctests print checks reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/diag.n.md, nodesrc/test_stdlib_diag_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T000515443Z-DIAG-DOCTESTS-PRINT-CHECKS-REPORTS-W-C859A6D5: diag doctests print checks reports without stdout fixtures

## 概要

stdlib/tests/diag.n.md has two runtime doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and still use legacy checks_* helpers.

## 対象

- `stdlib/tests/diag.n.md, nodesrc/test_stdlib_diag_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/diag.n.md` の 2 doctest は `diag_to_string` / `diags_to_string` の表示結果を検査していた。
- どちらも `checks_print_report` と `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。
- 診断表示は改行を含む文字列を扱うため、stdout fixture で escaping と expected / actual を固定しないと report format の退行を見落とす。

## 問題

stdlib/tests/diag.n.md has two runtime doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and still use legacy checks_* helpers.

## 影響

Diag rendering order and string escaping can regress without fixture-checked assertion labels and expected/actual stdout details, weakening diagnostics verification for compiler and stdlib work.

## 修正方針

Migrate both diag doctests to named TestReport stdout fixtures, add exit_code metadata, and add a source policy contract that rejects checks_* / ret-only regression.

## 検証

Run the diag source policy contract, focused diag doctests, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `diag_to_string_formats_structured_fields` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- `diags_to_string_keeps_order` も同じく stdout report + exit_code fixture へ移行した。
- 旧 `checks_*` helper を named `TestReport` API へ置き換え、改行を含む診断文字列の expected / actual を stdout 上で固定した。
- `nodesrc/test_stdlib_diag_nmd_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式への退行を source policy で拒否する。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
