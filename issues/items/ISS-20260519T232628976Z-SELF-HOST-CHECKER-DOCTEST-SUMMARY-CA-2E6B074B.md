---
id: ISS-20260519T232628976Z-SELF-HOST-CHECKER-DOCTEST-SUMMARY-CA-2E6B074B
title: "self-host checker doctest summary case exceeds default compile timeout"
area: selfhost
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-19
updated: 2026-05-20
target: "tests/stdlib/neplg2_checker.n.md; stdlib/neplg2/core/check/module.nepl; stdlib/neplg2/core/proof/solver.nepl; nepl-core static check compile performance"
---

# ISS-20260519T232628976Z-SELF-HOST-CHECKER-DOCTEST-SUMMARY-CA-2E6B074B: self-host checker doctest summary case exceeds default compile timeout

## 概要

`tests/stdlib/neplg2_checker.n.md::doctest#1` exceeded the default 60000ms wasm compile timeout locally. Re-running the suite with `NEPL_TEST_CASE_TIMEOUT_MS=120000` passed, and doctest#1 measured compile_ms around 61208ms, so the failure is compile-time budget pressure rather than runtime failure.

## 対象

- `tests/stdlib/neplg2_checker.n.md; stdlib/neplg2/core/check/module.nepl; stdlib/neplg2/core/proof/solver.nepl; nepl-core static check compile performance`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-pub-impl-checker-retry.json -j 1 --dist web/dist --assert-io` failed with `wasm test case timeout after 60000ms` at doctest#1 compile phase.
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='120000'; node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-pub-impl-checker-120s.json -j 1 --dist web/dist --assert-io` passed 5/5.
- The 120s run recorded doctest#1 `compile_ms` around 61208ms and run time around 23ms, so the immediate pressure is compile-time cost, not runtime execution.

## 問題

`tests/stdlib/neplg2_checker.n.md::doctest#1` exceeded the default 60000ms wasm compile timeout locally. Re-running the suite with `NEPL_TEST_CASE_TIMEOUT_MS=120000` passed, and doctest#1 measured compile_ms around 61208ms, so the failure is compile-time budget pressure rather than runtime failure.

## 影響

The broad checker summary doctest can intermittently fail local/CI verification near the default timeout boundary. This risks hiding real static-check regressions behind flaky timeout noise if the cause is not separated.

## 修正方針

Investigate whether the cause is the doctest's broad fixture shape, self-host checker/proof monomorphization cost, or compiler compile-time complexity. Prefer reducing proof/checker compile cost or splitting the broad doctest into focused cases; do not solve this by globally raising the timeout.

## 検証

`node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o <out>.json -j 1 --dist web/dist --assert-io` should pass with the default 60000ms timeout, and the report should record compile_ms comfortably below the threshold.

- 根本原因は、checker summary doctest 自体が `checks1` から `checks11` まで ownership accumulator を直列に再束縛し、同じ `TestReport` owner の push 結果を多数の中間 local として型検査させていたことだった。検査内容は必要だったが、test code shape が compile-time budget を圧迫していた。
- `tests/stdlib/neplg2_checker.n.md::summarizes_module_items_with_typed_kind_match` を mutable `checks` accumulator に変更し、11個の検査内容と stdout はそのまま維持した。
- timeout を延長する修正や検査内容の削除は行っていない。
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-checker-timeout-fix.json -j 1 --dist web/dist --assert-io`: 4/4 passed with the default 60000ms case timeout。
- 修正後の doctest#1 は `compile_ms` 55911ms / `total_ms` 55932ms で、default 60000ms を下回った。
