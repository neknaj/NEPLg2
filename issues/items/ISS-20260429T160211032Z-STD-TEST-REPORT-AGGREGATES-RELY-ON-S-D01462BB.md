---
id: ISS-20260429T160211032Z-STD-TEST-REPORT-AGGREGATES-RELY-ON-S-D01462BB
title: "std/test report aggregates rely on shallow Copy owner flow"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/test.nepl, tests/stdlib/std_test_collect.n.md, nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js"
---

# ISS-20260429T160211032Z-STD-TEST-REPORT-AGGREGATES-RELY-ON-S-D01462BB: std/test report aggregates rely on shallow Copy owner flow

## 概要

TestAssertion and TestReport kept str-bearing aggregate values Copy, so Resource IR could not force the report/ assertion owner boundary to be consumed exactly once. Quiet checks also constructed TestAssertion just to return Result<(),str>, creating report strings on success paths that did not need them.

## 対象

- `stdlib/std/test.nepl, tests/stdlib/std_test_collect.n.md, nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`

## 根拠

- GitHub Actions の `tutorials-test` 失敗を local で再現すると、`tutorials/getting_started/02_test_harness.n.md` が `checks_exit_code` の `TestReport` field owner leak で compile fail した。
- `tests/stdlib/std_test_collect.n.md` は `test_report_print_stdout report` 後の `shown` / `report` field owner が残り、`TestAssertion` / `TestReport` の浅い `Copy` が最終消費境界を曖昧にしていた。
- `check_eq_i32` / `check_str_eq` / `check_ok_i32` / `check_err_i32` は quiet `Result<(),str>` を返すだけなのに `TestAssertion` を中間生成しており、成功時にも report 用 `str` owner を作る余地があった。

## 問題

TestAssertion and TestReport kept str-bearing aggregate values Copy, so Resource IR could not force the report/ assertion owner boundary to be consumed exactly once. Quiet checks also constructed TestAssertion just to return Result<(),str>, creating report strings on success paths that did not need them.

## 影響

tutorials and std/test report fixtures failed under the strict Resource IR owner gate, and shallow aggregate Copy could hide duplicated report string ownership instead of exposing the real terminal boundary.

## 修正方針

Remove Copy/Clone for TestAssertion and TestReport, make render/status helpers observe by reference, consume assertion/report values explicitly at push/exit boundaries, and keep quiet check helpers from allocating TestAssertion values.

## 検証

- `node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-tree -o tmp/std-test-owner-boundary-3.json -j 1 --dist web/dist`: total=3 passed=3
- `node nodesrc/tests.js -i tutorials/getting_started/02_test_harness.n.md --no-tree -o tmp/tutorial-02-owner-boundary-2.json -j 1 --dist web/dist`: total=1 passed=1
- `node nodesrc/tests.js -i tutorials --no-tree -o tmp/tutorials-owner-boundary-1.json -j 4 --dist web/dist`: total=24 passed=19 failed=5。`std/test` quiet harness failure は解消し、残りは Vec / ByteBuilder / generic Result owner の別 issue 領域。
