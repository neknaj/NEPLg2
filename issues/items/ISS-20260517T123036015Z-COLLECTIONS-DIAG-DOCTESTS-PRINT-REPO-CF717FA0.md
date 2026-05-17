---
id: ISS-20260517T123036015Z-COLLECTIONS-DIAG-DOCTESTS-PRINT-REPO-CF717FA0
title: "collections diag doctests print reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: tests/stdlib/collections_diag.n.md
---

# ISS-20260517T123036015Z-COLLECTIONS-DIAG-DOCTESTS-PRINT-REPO-CF717FA0: collections diag doctests print reports without stdout fixtures

## 概要

tests/stdlib/collections_diag.n.md calls checks_print_report in four std/test doctests but leaves stdout and exit_code metadata unspecified, so report format regressions are not fixture-checked.

## 対象

- `tests/stdlib/collections_diag.n.md`

## 根拠

- `tests/stdlib/collections_diag.n.md` の 4 件はいずれも `std/test` を import し、`checks_print_report checks` で assertion report を stdout へ出力していた。
- しかし manifest に `stdout:` と `exit_code:` がなく、runner は report stdout の内容を fixture として比較していなかった。
- focused 実行では 4 件とも `Checked [ok]\n[0] ok\n` を出しており、既に出力している report を期待値へ固定できる状態だった。

## 問題

tests/stdlib/collections_diag.n.md calls checks_print_report in four std/test doctests but leaves stdout and exit_code metadata unspecified, so report format regressions are not fixture-checked.

## 影響

The doctests can keep passing with only exit status/return behavior even if the assertion report text changes or disappears, weakening the stdout assertion policy needed for selfhost shared .n.md tests.

## 修正方針

Add stdio/normalize_newlines tags, deterministic stdout metadata, and exit_code metadata to all four collections_diag doctests while preserving the existing Diag/Option behavior checks.

## 検証

Run focused collections_diag doctests through nodesrc/tests.js with --assert-io and run the std/test report contract/source policy checks.

## 対応内容

- `tests/stdlib/collections_diag.n.md` の 4 件を `neplg2:test[stdio, normalize_newlines]` に変更し、`stdout: "Checked [ok]\n[0] ok\n"` と `exit_code: 0` を追加した。
- `checks_print_report checks` で表示してから `checks_exit_code shown` を返す既存の検査順序は維持した。
- `nodesrc/test_stdlib_collections_diag_report_contract.js` を追加し、4 件が `ret:` へ戻らず、stdout report と exit code を固定し続けることを parser-level で監視する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。

## 検証結果

- `node nodesrc/test_stdlib_collections_diag_report_contract.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/agent1-collections-diag-report-tests.json -j 2 --assert-io --dist web/dist`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
