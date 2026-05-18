---
id: ISS-20260518T224806912Z-SELFHOST-CLI-FILE-IO-DOCTESTS-HIDE-S-3BA5E109
title: "selfhost cli file_io doctests hide reports behind ret metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_cli_file_io.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T224806912Z-SELFHOST-CLI-FILE-IO-DOCTESTS-HIDE-S-3BA5E109: selfhost cli file_io doctests hide reports behind ret metadata

## 概要

Three selfhost_cli_file_io doctests build std/test Checks but return checks_exit_code directly under ret: 0, and the root source read doctest also reports only numeric status. File I/O behavior therefore does not appear in stdout fixtures.

## 対象

- `tests/stdlib/selfhost_cli_file_io.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/selfhost_cli_file_io.n.md` の doctest は file count、read failure diagnostic、text artifact、binary artifact を検査しているが、manifest は `ret: 0` 中心で stdout expectation を固定していなかった。
- `std/test::Checks` を使う 3 件は `checks_exit_code checks` を直接返しており、成功時の assertion report が表示されていなかった。

## 問題

Three selfhost_cli_file_io doctests build std/test Checks but return checks_exit_code directly under ret: 0, and the root source read doctest also reports only numeric status. File I/O behavior therefore does not appear in stdout fixtures.

## 影響

Selfhost file I/O runner requirements can match exit status while losing report detail for read failure and artifact write behavior.

## 修正方針

Print Checks reports, migrate the four file_io doctests to stdio + exit_code + deterministic stdout, and add a source policy contract.

## 修正内容

- `tests/stdlib/selfhost_cli_file_io.n.md` の 4 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- root source read case も `file_count == 1` を `std/test::Checks` report として出すようにし、単なる numeric status ではなく観測内容を stdout fixture に残した。
- 既存の diagnostic / text artifact / binary artifact case は `checks_print_report` の結果を `checks_exit_code` へ渡す形にした。
- `nodesrc/test_selfhost_cli_file_io_report_contract.js` を追加し、`ret:` 再導入、stdout fixture 欠落、report 出力なしの exit-code-only 退行を source policy で拒否する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_selfhost_cli_file_io_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_file_io.n.md --no-tree -o tmp/agent1-selfhost-cli-file-io-report.json -j 1 --dist web/dist --assert-io`
