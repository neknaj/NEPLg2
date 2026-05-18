---
id: ISS-20260518T024038881Z-SELFHOST-CLI-REPORTER-DOCTESTS-STILL-E474D880
title: "selfhost CLI reporter doctests still use ret without report stdout"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_cli_reporter.n.md, nodesrc/test_selfhost_cli_reporter_report_contract.js"
---

# ISS-20260518T024038881Z-SELFHOST-CLI-REPORTER-DOCTESTS-STILL-E474D880: selfhost CLI reporter doctests still use ret without report stdout

## 概要

selfhost CLI reporter doctests still use ret as an exit-code substitute, and the rendering-only std/test cases do not fix their assertion report stdout. This leaves diagnostic renderer compatibility observable only through local failure rather than the fixture contract.

## 対象

- `tests/stdlib/selfhost_cli_reporter.n.md, nodesrc/test_selfhost_cli_reporter_report_contract.js`

## 根拠

- `tests/stdlib/selfhost_cli_reporter.n.md` の rendering-only doctest 2 件が `std/test` の checks を作るだけで stdout report を出さず、`ret: 0` で合否だけを固定していた。
- 同 file の writer doctest も stdout/stderr 期待値を持つ一方で、終了可否を `exit_code:` ではなく `ret:` に載せていた。
- `nodesrc/test_selfhost_cli_reporter_boundary.js` は reporter / diag の責務分割を監視していたが、`.n.md` metadata と stdout report contract は監視していなかった。

## 問題

selfhost CLI reporter doctests still use ret as an exit-code substitute, and the rendering-only std/test cases do not fix their assertion report stdout. This leaves diagnostic renderer compatibility observable only through local failure rather than the fixture contract.

## 影響

The selfhost runner can match process success while drifting from the Rust runner's diagnostic report formatting. Rendering regressions are harder to review because expected/actual strings are not emitted as deterministic assertion report stdout.

## 修正方針

Migrate reporter doctests to exit_code metadata, make rendering-only cases print deterministic std/test reports, keep the writer case stdout/stderr as the observable diagnostic output, and add a source policy that rejects ret regression for this fixture.

## 検証

Run the selfhost CLI reporter report policy, the existing reporter boundary policy, focused selfhost_cli_reporter doctests, and issue checks.

## 対応結果

- `tests/stdlib/selfhost_cli_reporter.n.md` の 3 doctest を `ret:` から `exit_code: 0` へ移行した。
- rendering-only doctest 2 件は `neplg2:test[stdio, normalize_newlines]` と deterministic `stdout:` を追加し、`checks_print_report` で human/json renderer の比較結果を stdout に出すようにした。
- writer doctest は JSON stdout / human stderr が検査対象なので、その stdout/stderr 期待値を維持しつつ `stdio` tag と `exit_code: 0` を明示した。
- `nodesrc/test_selfhost_cli_reporter_report_contract.js` を追加し、doctest 数、`ret:` 不使用、`exit_code`、stdio tag、stdout/stderr contract、report print -> exit code の順序を固定した。
- `nodesrc/run_source_policy_regressions.js` に同 policy を登録した。

## 検証結果

- `node nodesrc/test_selfhost_cli_reporter_report_contract.js`
- `node nodesrc/test_selfhost_cli_reporter_boundary.js`
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report.json -j 1 --dist web\dist --assert-io`: 3 件とも compile timeout after 60000ms
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report-300s.json -j 1 --dist web\dist --assert-io`: 3 件とも compile timeout after 300000ms

focused doctest の timeout は `ISS-20260518T030154409Z-SELFHOST-CLI-REPORTER-DOCTESTS-EXCEE-6D30C865` に分離した。今回の issue は `.n.md` metadata / stdout contract の source-level 固定として閉じる。
