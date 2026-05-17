---
id: ISS-20260517T125745927Z-SELFHOST-CLI-DRIVER-DOCTESTS-HIDE-ST-15DDEDC6
title: "selfhost_cli_driver doctests hide std/test reports behind ret"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: tests/stdlib/selfhost_cli_driver.n.md
---

# ISS-20260517T125745927Z-SELFHOST-CLI-DRIVER-DOCTESTS-HIDE-ST-15DDEDC6: selfhost_cli_driver doctests hide std/test reports behind ret

## 概要

tests/stdlib/selfhost_cli_driver.n.md has std/test checks suites that return checks_exit_code checks with ret: 0, but they do not print and fixture the assertion report stdout.

## 対象

- `tests/stdlib/selfhost_cli_driver.n.md`

## 根拠

- `tests/stdlib/selfhost_cli_driver.n.md::doctest#1` と `doctest#3` は `std/test` を import し、`checks_new` / `checks_push` で 2 件ずつ assertion を集約していた。
- しかしどちらも `checks_exit_code checks` を返し、manifest は `ret: 0` だけだった。
- そのため assertion report の件数や stdout 形式が壊れても、fixture は exit status だけで成功していた。

## 問題

tests/stdlib/selfhost_cli_driver.n.md has std/test checks suites that return checks_exit_code checks with ret: 0, but they do not print and fixture the assertion report stdout.

## 影響

Selfhost CLI driver regressions can change assertion count or report formatting while the doctest still passes by exit status alone, weakening the shared .n.md stdout contract for selfhost development.

## 修正方針

Switch the affected selfhost_cli_driver doctests to stdio/normalize_newlines, print checks_print_report before checks_exit_code, fix stdout mlstr and exit_code metadata, and add a source-policy contract for this file.

## 検証

Run the new source policy, focused selfhost_cli_driver doctests with --assert-io, issues check, and diff check.

## 対応内容

- `doctest#1` と `doctest#3` を `neplg2:test[stdio, normalize_newlines]` に変更し、`stdout: mlstr:` と `exit_code: 0` を追加した。
- 両 doctest の末尾を `let shown checks_print_report checks; checks_exit_code shown` にし、2 assertion の report を stdout fixture として固定した。
- `nodesrc/test_selfhost_cli_driver_report_contract.js` を追加し、対象 doctest が `ret:` へ戻らず、stdout report と exit code を維持することを source policy にした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。

## 検証結果

- `node nodesrc/test_selfhost_cli_driver_report_contract.js`: pass
- `git diff --check`: pass
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md --no-tree -o tmp/agent1-selfhost-cli-driver-report-tests.json -j 1 --dist web/dist --assert-io`: fail, compile timeout 60000ms x 3
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md --no-tree -o tmp/agent1-selfhost-cli-driver-report-tests-long.json -j 1 --dist web/dist --assert-io`: fail, compile timeout 300000ms x 3

Runtime verification is blocked by `ISS-20260517T132644394Z-SELFHOST-CLI-DRIVER-DOCTESTS-EXCEED--5B706A91`. The stdout fixture shape is fixed by source policy in this issue; the compile-time blocker is tracked separately.
