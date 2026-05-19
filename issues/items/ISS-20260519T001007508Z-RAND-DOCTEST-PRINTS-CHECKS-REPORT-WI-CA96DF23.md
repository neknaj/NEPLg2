---
id: ISS-20260519T001007508Z-RAND-DOCTEST-PRINTS-CHECKS-REPORT-WI-CA96DF23
title: "rand doctest prints checks report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/rand.n.md, nodesrc/test_stdlib_rand_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T001007508Z-RAND-DOCTEST-PRINTS-CHECKS-REPORT-WI-CA96DF23: rand doctest prints checks report without stdout fixture

## 概要

stdlib/tests/rand.n.md calls checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata and still uses legacy checks_* helpers.

## 対象

- `stdlib/tests/rand.n.md, nodesrc/test_stdlib_rand_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/rand.n.md` は xorshift32 の deterministic state progression を検査していた。
- 旧実装は `checks_print_report` / `checks_exit_code` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。
- 検査内容も `check_ne eq ... true` という読みにくい形で、どの state 条件を見ているかが stdout fixture に残っていなかった。

## 問題

stdlib/tests/rand.n.md calls checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata and still uses legacy checks_* helpers.

## 影響

The deterministic xorshift state progression can regress without fixture-checked assertion labels for nonzero and changing states.

## 修正方針

Migrate the rand doctest to named TestReport stdout fixture, add exit_code metadata, and add a source policy contract that rejects checks_* / ret-only regression.

## 検証

Run the rand source policy contract, focused rand doctest, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `rand_main` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- 旧 `checks_*` helper を named `TestReport` API へ置き換えた。
- first / second generated state の nonzero、successive state difference、zero seed escape の4観測を assertion label として stdout に固定した。
- `nodesrc/test_stdlib_rand_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式への退行を source policy で拒否する。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
