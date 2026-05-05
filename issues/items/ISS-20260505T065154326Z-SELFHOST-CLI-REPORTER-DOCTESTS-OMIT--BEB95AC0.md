---
id: ISS-20260505T065154326Z-SELFHOST-CLI-REPORTER-DOCTESTS-OMIT--BEB95AC0
title: "selfhost CLI reporter doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/selfhost_cli_reporter.n.md
---

# ISS-20260505T065154326Z-SELFHOST-CLI-REPORTER-DOCTESTS-OMIT--BEB95AC0: selfhost CLI reporter doctests omit stdout assertion reports

## 概要

selfhost_cli_reporter の assertion-style doctests が std/test checks を作る一方で checks_print_report を呼ばず、ret: 0 だけで成功を表している。

## 対象

- `tests/stdlib/selfhost_cli_reporter.n.md`

## 根拠

- `tests/stdlib/selfhost_cli_reporter.n.md` の `selfhost_cli_reporter_renders_single_human_and_json` と `selfhost_cli_reporter_renders_collection_human_and_json` は `std/test` の `checks_push` で assertion suite を作るが、`checks_print_report` を呼ばず `ret: 0` だけで成功を表していた。
- `selfhost_cli_reporter_writes_json_stdout_and_human_stderr` は stdout/stderr fixture を持つが、process success を `ret: 0` で表しており、`.n.md` runner contract 上の exit code と言語戻り値が混ざっていた。

## 問題

selfhost_cli_reporter の assertion-style doctests が std/test checks を作る一方で checks_print_report を呼ばず、ret: 0 だけで成功を表している。

## 影響

selfhost CLI diagnostics reporter の human/json rendering regression が stdout fixture に固定されず、Rust runner と selfhost runner の assertion report contract を比較できない。

## 修正方針

std/test checks を checks_print_report に通し、stdout に deterministic report を固定する。process success は ret: ではなく exit_code: で表す。

## 検証

tests/stdlib/selfhost_cli_reporter.n.md を focused run し、stdout/stderr と exit_code expectation が通ることを確認する。

## 対応

- assertion-style doctest 2 件に `checks_print_report` を追加し、`Checked [ok,ok]` の stdout report を fixture として固定した。
- 3 件すべての process success metadata を `ret: 0` から `exit_code: 0` へ移行した。
- writer doctest は既存の stdout/stderr fixture を維持し、JSON stdout と human stderr の契約をそのまま検証する。

## 2026-05-05 検証結果

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_reporter.n.md --no-tree -o tmp/selfhost-cli-reporter-report-agent1.json -j 1 --dist web/dist`: total=3, passed=3, failed=0
