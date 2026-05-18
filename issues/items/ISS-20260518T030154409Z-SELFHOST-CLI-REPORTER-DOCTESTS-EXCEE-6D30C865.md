---
id: ISS-20260518T030154409Z-SELFHOST-CLI-REPORTER-DOCTESTS-EXCEE-6D30C865
title: "selfhost CLI reporter doctests exceed local compile timeout"
area: TEST
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_cli_reporter.n.md, stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/core/infra/diag.nepl, nepl-core/src/resource"
---

# ISS-20260518T030154409Z-SELFHOST-CLI-REPORTER-DOCTESTS-EXCEE-6D30C865: selfhost CLI reporter doctests exceed local compile timeout

## 概要

Focused selfhost CLI reporter doctests time out during compile even with NEPL_TEST_CASE_TIMEOUT_MS=300000. The timeout happens before run output is produced and affects all three reporter doctests.

## 対象

- `tests/stdlib/selfhost_cli_reporter.n.md, stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/core/infra/diag.nepl, nepl-core/src/resource`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report.json -j 1 --dist web\dist --assert-io` は 3 doctest すべてで `wasm test case timeout after 60000ms` になった。
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report-300s.json -j 1 --dist web\dist --assert-io` でも 3 doctest すべてが `wasm test case timeout after 300000ms` になった。
- timeout の `last_phase` は `compile` で、run output 生成前に止まっている。

## 問題

Focused selfhost CLI reporter doctests time out during compile even with NEPL_TEST_CASE_TIMEOUT_MS=300000. The timeout happens before run output is produced and affects all three reporter doctests.

## 影響

Reporter fixture changes cannot be locally validated by focused doctest execution, and CI may spend excessive time on selfhost diagnostic rendering cases. The cause may be compiler/static-check cost in the selfhost diagnostic import graph rather than the generated wasm runtime.

## 修正方針

Profile the selfhost CLI reporter doctest compile path, identify whether Resource IR/static-check summary construction, module loading, monomorphization, or selfhost diagnostic dependencies dominate, then fix the compiler/static-check algorithm or split fixture dependencies without weakening diagnostic coverage.

## 検証

Re-run tests/stdlib/selfhost_cli_reporter.n.md with the default 60000ms case timeout and assert-io enabled; record compile_ms/run_ms once it completes.
