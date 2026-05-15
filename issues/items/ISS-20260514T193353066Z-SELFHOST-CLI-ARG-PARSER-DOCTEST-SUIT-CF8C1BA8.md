---
id: ISS-20260514T193353066Z-SELFHOST-CLI-ARG-PARSER-DOCTEST-SUIT-CF8C1BA8
title: "selfhost CLI arg parser doctest suite repeats expensive compile work"
area: test
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-14
updated: 2026-05-15
target: "tests/stdlib/selfhost_cliarg_parser.n.md, nodesrc/test_selfhost_cliarg_parser_doctest_contract.js, nodesrc/tests.js"
---

# ISS-20260514T193353066Z-SELFHOST-CLI-ARG-PARSER-DOCTEST-SUIT-CF8C1BA8: selfhost CLI arg parser doctest suite repeats expensive compile work

## 概要

Focused verification of tests/stdlib/selfhost_cliarg_parser.n.md passes, but the 10 doctests take about 130s locally because each case recompiles the same selfhost CLI args dependency graph. The JSON timing shows runtime is only 5-10ms per case while compile_ms is about 10-17s per case.

## 対象

- `tests/stdlib/selfhost_cliarg_parser.n.md, nodesrc/tests.js, nepl-core`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/agent1-selfhost-cli-args-vec-observer-tests-final.json -j 1 --dist web/dist --assert-io` は total=10, passed=10 で通過したが、PowerShell `Measure-Command` で約 129.7 秒かかった。
- 出力 JSON の各 case は `run_ms` が 5-10ms 程度で、`compile_ms` が約 10-17s に集中している。
- 並列確認中の 120 秒 command timeout では suite 完了前に外側 command が終了したため、local focused verification の運用上も影響がある。

## 問題

Focused verification of tests/stdlib/selfhost_cliarg_parser.n.md passes, but the 10 doctests take about 130s locally because each case recompiles the same selfhost CLI args dependency graph. The JSON timing shows runtime is only 5-10ms per case while compile_ms is about 10-17s per case.

## 影響

Selfhost CLI parser regressions are correct but expensive to verify. Agents may hit command-level timeouts or skip the suite, and static-check performance regressions can be confused with generated wasm runtime behavior unless compile-time timing is tracked.

## 修正方針

Investigate whether nodesrc/tests.js can reuse loaded compiler/module state for same-file doctests, or whether the selfhost CLI args dependency graph should be split so small parser cases avoid compiling driver/reporting/JSON-heavy modules. Do not raise budgets as the primary fix; first determine compiler/static-check cost and dependency shape.

## 検証

Run tests/stdlib/selfhost_cliarg_parser.n.md with timing JSON and confirm total time and per-case compile_ms improve without weakening Resource IR, type, owner, lifetime, or effect checks.

## 2026-05-15 Agent 1 対応

調査の結果、この suite の支配要因は generated wasm runtime ではなく compile phase だった。各 case の `run_ms` は数 ms だが、同じ `neplg2/cli/args` dependency graph を 10 doctest で毎回 compile していたため、focused run 全体が 120 秒を超えていた。

対応として、`tests/stdlib/selfhost_cliarg_parser.n.md` を 10 個の独立 doctest から 1 個の aggregated doctest に再構成した。各 parser behavior は helper function として分離し、`std/test` の canonical `TestReport` に 10 本の assertion label として集約する。これにより検査意味は削らず、stdout report と `exit_code: 0` によって失敗箇所も fixture から読める。

再発防止として `nodesrc/test_selfhost_cliarg_parser_doctest_contract.js` を追加し、この suite が 1 doctest、`stdio` / `normalize_newlines`、`exit_code` metadata、canonical `test_report_print_stdout` / `test_report_exit_code` 構成を保つことを監視する。`ret:` に戻す退行も拒否する。

検証:

- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cliarg_parser.n.md -n 1 --assert-io --dist web/dist`: pass, `compile_ms=18793`, `total_ms=18837`
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/agent1-selfhost-cliarg-parser-aggregated.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/test_selfhost_cliarg_parser_doctest_contract.js`
