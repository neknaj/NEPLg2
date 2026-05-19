---
id: ISS-20260519T015240790Z-N-MD-REPORT-METADATA-POLICY-MISSES-S-8E4C48FA
title: ".n.md report metadata policy misses stdout/exit_code regressions"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-19
target: "tests/**/*.n.md, tutorials/**/*.n.md, stdlib/**/*.n.md, examples/**/*.n.md, nodesrc/run_source_policy_regressions.js"
---

# ISS-20260519T015240790Z-N-MD-REPORT-METADATA-POLICY-MISSES-S-8E4C48FA: .n.md report metadata policy misses stdout/exit_code regressions

## 概要

Report-style .n.md doctests can call checks_print_report/test_report_print_stdout and checks_exit_code/test_report_exit_code while omitting stdout, exit_code, stdio, or normalize_newlines metadata. Existing per-file contracts do not provide a general safety net.

## 対象

- `tests/**/*.n.md, tutorials/**/*.n.md, stdlib/**/*.n.md, examples/**/*.n.md, nodesrc/run_source_policy_regressions.js`

## 根拠

- `node` による parser scan で、report helper を呼ぶ `.n.md` doctest 444 件のうち 16 件が一般 policy では検出されていなかった。
- 既存の per-file contract は migrated file の退行を固定できるが、未登録の `.n.md` が `stdout:` / `exit_code:` / `stdio` / `normalize_newlines` を落としても横断的には検出できなかった。
- `tests/compiler/overload.n.md` では stdout report を固定済みの 15 doctest が `exit_code:` だけ欠いていた。
- `tests/stdlib/stdio_read_all.n.md` では stdout / exit_code を固定済みの doctest が `stdio, normalize_newlines` tag を欠いていた。

## 問題

Report-style .n.md doctests can call checks_print_report/test_report_print_stdout and checks_exit_code/test_report_exit_code while omitting stdout, exit_code, stdio, or normalize_newlines metadata. Existing per-file contracts do not provide a general safety net.

## 影響

A migrated doctest can regress to an exit-code-only or unpinned stdout style without failing source policy, weakening selfhost runner compatibility and assertion-report diagnosability.

## 修正方針

Add a repository-wide .n.md source policy that scans active report doctests and requires report print plus report-derived exit code, stdio plus normalize_newlines tags, pinned stdout, pinned exit_code, and no ret metadata. Fix existing .n.md metadata gaps so the policy is enforceable without baselines.

## 検証

node nodesrc/test_nmd_report_metadata_policy.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check; git diff --check

## 対応結果

- `nodesrc/test_nmd_report_metadata_policy.js` を追加し、`.n.md` の active report doctest を横断して検査するようにした。
- policy は `checks_print_report` / `test_report_print_stdout` と `checks_exit_code` / `test_report_exit_code` の片側だけの使用、`stdout:` 欠落、`exit_code:` 欠落、`ret:` 併用、`stdio` / `normalize_newlines` tag 欠落を拒否する。
- `tests/compiler/overload.n.md` の 15 doctest に `exit_code: 0` を追加し、report stdout と exit code expectation を分離した。
- `tests/stdlib/stdio_read_all.n.md` の report doctest に `stdio, normalize_newlines` tag を追加し、stdout fixture の改行正規化を manifest 上も明示した。
- `nodesrc/run_source_policy_regressions.js` に新 policy を登録した。

確認済み:

- `node nodesrc/test_nmd_report_metadata_policy.js`
- `node nodesrc/run_doctest.js -i tests\compiler\overload.n.md -n 8 --dist web\dist`
- `node nodesrc/run_doctest.js -i tests\stdlib\stdio_read_all.n.md -n 2 --dist web\dist`
- `node nodesrc/run_source_policy_regressions.js --warn-only`

制限:

- `node nodesrc/tests.js -i tests\compiler\overload.n.md -i tests\stdlib\stdio_read_all.n.md --no-tree -o tmp\agent1-nmd-report-metadata-policy.json -j 1 --dist web\dist --assert-io` は 300 秒で timeout した。`overload.n.md` 全体はコンパイル時間が大きいため、local では代表 doctest と source policy に絞って確認した。
