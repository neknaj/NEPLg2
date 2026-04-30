---
id: ISS-20260430T125244690Z-STDLIB-RESULT-DOCTEST-USES-STD-TEST--F99DF5C9
title: "stdlib result doctest uses std/test without stdout report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/result.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T125244690Z-STDLIB-RESULT-DOCTEST-USES-STD-TEST--F99DF5C9: stdlib result doctest uses std/test without stdout report

## 概要

stdlib/tests/result.n.md imports std/test and aggregates assertions, but returns checks_exit_code directly with ret: 0 metadata and never prints the assertion report to stdout.

## 対象

- `stdlib/tests/result.n.md`

## 根拠

- `stdlib/tests/result.n.md` は `#import "std/test" as *` し、13件の assertion を `checks_push` で集約していた。
- しかし末尾は `checks_exit_code checks` だけで、`checks_print_report` を呼ばないため stdout に assertion report が出ていなかった。
- metadata も `ret: 0` のままで、process success/failure と言語戻り値検証の意味が混ざっていた。

## 問題

stdlib/tests/result.n.md imports std/test and aggregates assertions, but returns checks_exit_code directly with ret: 0 metadata and never prints the assertion report to stdout.

## 影響

The Result doctest only exposes a success exit value, so self-host and Rust runners cannot compare assertion report details for Result APIs.

## 修正方針

Call checks_print_report, return checks_exit_code of the shown report, replace ret: 0 with exit_code: 0, and pin stdout to the deterministic assertion report.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/result.n.md --no-tree -o tmp/stdlib-result-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`stdlib/tests/result.n.md` を stdout report + `exit_code: 0` 形式へ移行した。`main` は `checks_print_report checks` の結果を `shown` に束縛し、`checks_exit_code shown` を返す。stdout fixture には13件の assertion reportを固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/result.n.md --no-tree -o tmp/stdlib-result-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1
