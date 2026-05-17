---
id: ISS-20260517T190658373Z-SELFHOST-DIAG-OUTCOME-DOCTESTS-HIDE--A0C5B813
title: "selfhost diag outcome doctests hide std/test reports in metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-18
target: "tests/stdlib/neplg2_diag_outcome.n.md, nodesrc/test_selfhost_diag_outcome_report_contract.js"
---

# ISS-20260517T190658373Z-SELFHOST-DIAG-OUTCOME-DOCTESTS-HIDE--A0C5B813: selfhost diag outcome doctests hide std/test reports in metadata

## 概要

tests/stdlib/neplg2_diag_outcome.n.md doctest#1 and #2 call checks_print_report and checks_exit_code, but their manifests still use bare neplg2:test metadata without stdout and exit_code expectations. The assertion report is emitted at runtime but is not part of the fixture contract.

## 対象

- `tests/stdlib/neplg2_diag_outcome.n.md, nodesrc/test_selfhost_diag_outcome_report_contract.js`

## 根拠

- `tests/stdlib/neplg2_diag_outcome.n.md::doctest#1` と `doctest#2` は `checks_print_report` で8件ずつの assertion report をstdoutへ出していた。
- どちらも manifest は bare `neplg2:test` で、`stdio` / `normalize_newlines` tag、`exit_code: 0`、`stdout:` expectation を持っていなかった。
- focused run では両方とも `Checked [ok,ok,ok,ok,ok,ok,ok,ok]` を出していたため、runnerはreportを観測できるのにfixture契約として固定していなかった。

## 問題

tests/stdlib/neplg2_diag_outcome.n.md doctest#1 and #2 call checks_print_report and checks_exit_code, but their manifests still use bare neplg2:test metadata without stdout and exit_code expectations. The assertion report is emitted at runtime but is not part of the fixture contract.

## 影響

Self-host diagnostic and Outcome regressions can change report count, ordering, or printed assertion details while the doctest still passes by return value only. This keeps .n.md runner semantics ambiguous and weakens self-host parity checks.

## 修正方針

Move the two doctests to neplg2:test[stdio, normalize_newlines], add exit_code: 0 and deterministic stdout report expectations, and add a source policy contract that rejects ret-only or stdout-less metadata for this file.

## 対応内容

- `doctest#1` と `doctest#2` を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- 既存の `checks_print_report` -> `checks_exit_code` の実行順は維持し、テスト本体の検査ロジックは変更していない。
- `nodesrc/test_selfhost_diag_outcome_report_contract.js` を追加し、3件の doctest の metadata contract を固定した。
- `nodesrc/run_source_policy_regressions.js` へ新しい contract を登録した。
- 親 issue `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` に進捗を追記した。

## 検証

- `node nodesrc/test_selfhost_diag_outcome_report_contract.js`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_diag_outcome.n.md -n 1 --assert-io --dist web\dist`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_diag_outcome.n.md -n 2 --assert-io --dist web\dist`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_diag_outcome.n.md -n 3 --assert-io --dist web\dist`: passed
- `node nodesrc/tests.js -i tests\stdlib\neplg2_diag_outcome.n.md --no-tree -o tmp\agent1-neplg2-diag-outcome-report-metadata.json -j 1 --dist web\dist --assert-io`: total=3, passed=3
- `node nodesrc/issues.js check --dir issues`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `git diff --check`: CRLF warnings only
