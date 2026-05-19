---
id: ISS-20260519T011047746Z-STDLIB-JSON-DOCTEST-PRINTS-CHECKS-RE-3C7D519B
title: "stdlib json doctest prints checks report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/tests/json.n.md, nodesrc/test_stdlib_json_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T011047746Z-STDLIB-JSON-DOCTEST-PRINTS-CHECKS-RE-3C7D519B: stdlib json doctest prints checks report without stdout fixture

## 概要

stdlib/tests/json.n.md calls checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata.

## 対象

- `stdlib/tests/json.n.md, nodesrc/test_stdlib_json_nmd_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/json.n.md` は `JsonValue` の null / bool / number / string / array / object constructor と accessor を 13 assertion で検査していた。
- 旧 manifest は `neplg2:test` のみで、`checks_print_report` / `checks_exit_code` を呼んでいるにもかかわらず `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` を固定していなかった。
- JSON value model は selfhost 側でも report / config / structured output の基礎になるため、単なる exit status ではなく assertion report の互換性を fixture に残す必要がある。

## 問題

stdlib/tests/json.n.md calls checks_print_report and checks_exit_code, but the manifest does not pin stdout / exit_code metadata.

## 影響

JsonValue constructor and accessor regressions can agree on exit status while losing assertion report compatibility between Rust and selfhost runners.

## 修正方針

Pin json_main with stdio normalized stdout, exit_code: 0, and add a source policy contract rejecting ret-only or missing stdout report regression.

## 検証

Run the JSON source policy contract, focused json.n.md doctest with --assert-io, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `json_main` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- 既存の JSON constructor / accessor 13 assertion の順序と件数を stdout に固定した。
- `nodesrc/test_stdlib_json_nmd_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、JSON constructor / accessor assertion の欠落を source policy で拒否するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
