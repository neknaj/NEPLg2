---
id: ISS-20260517T124048715Z-FEATURES-TUI-BOX-HELPER-DOCTEST-HIDE-22D23F90
title: "features_tui box helper doctest hides std/test report behind ret"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: tests/stdlib/features_tui.n.md
---

# ISS-20260517T124048715Z-FEATURES-TUI-BOX-HELPER-DOCTEST-HIDE-22D23F90: features_tui box helper doctest hides std/test report behind ret

## 概要

tests/stdlib/features_tui.n.md doctest#4 imports std/test and builds a checks suite, but returns checks_exit_code checks with ret: 0 and never prints the report stdout.

## 対象

- `tests/stdlib/features_tui.n.md`

## 根拠

- `tests/stdlib/features_tui.n.md::doctest#4` は `std/test` を import し、15 件の `assert_str_eq` を `checks_push` で集約していた。
- しかし末尾は `checks_exit_code checks` だけで、`checks_print_report` を呼ばず、manifest も `ret: 0` のみだった。
- そのため assertion count や report stdout の形式が変わっても、fixture は stdout diff として検出できなかった。

## 問題

tests/stdlib/features_tui.n.md doctest#4 imports std/test and builds a checks suite, but returns checks_exit_code checks with ret: 0 and never prints the report stdout.

## 影響

The TUI box helper boundary can regress in assertion count or report output while the fixture only checks the process return value, weakening shared .n.md stdout contract coverage.

## 修正方針

Print the checks report, switch the doctest to stdio/normalize_newlines with deterministic stdout and exit_code metadata, and add a source policy contract for this fixture.

## 検証

Run the features_tui focused doctest with --assert-io, the new policy, issues check, and diff check.

## 対応内容

- 対象 doctest を `neplg2:test[stdio, normalize_newlines]` に変更し、`stdout: mlstr:` と `exit_code: 0` を追加した。
- `checks_print_report checks` の結果を `shown` に束縛してから `checks_exit_code shown` を返すようにし、15 assertion の report を stdout fixture として固定した。
- `nodesrc/test_features_tui_report_contract.js` を追加し、対象 doctest が `ret:` へ戻らず、stdout report と exit code を維持することを source policy にした。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。

## 検証結果

- `node nodesrc/test_features_tui_report_contract.js`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 4 --dist web/dist`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
