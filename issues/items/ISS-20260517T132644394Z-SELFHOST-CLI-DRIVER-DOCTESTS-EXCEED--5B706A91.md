---
id: ISS-20260517T132644394Z-SELFHOST-CLI-DRIVER-DOCTESTS-EXCEED--5B706A91
title: "selfhost_cli_driver doctests exceed extended compile timeout"
area: TEST
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-17
updated: 2026-05-17
target: "tests/stdlib/selfhost_cli_driver.n.md, nodesrc/tests.js"
---

# ISS-20260517T132644394Z-SELFHOST-CLI-DRIVER-DOCTESTS-EXCEED--5B706A91: selfhost_cli_driver doctests exceed extended compile timeout

## 概要

tests/stdlib/selfhost_cli_driver.n.md currently times out in compile phase for all three doctests even with NEPL_TEST_CASE_TIMEOUT_MS=300000. This blocks runtime verification of stdout fixture changes and indicates a selfhost fixture compile-time regression or an overly heavy driver test shape.

## 対象

- `tests/stdlib/selfhost_cli_driver.n.md, nodesrc/tests.js`

## 根拠

- 未記入

## 問題

tests/stdlib/selfhost_cli_driver.n.md currently times out in compile phase for all three doctests even with NEPL_TEST_CASE_TIMEOUT_MS=300000. This blocks runtime verification of stdout fixture changes and indicates a selfhost fixture compile-time regression or an overly heavy driver test shape.

## 影響

Selfhost CLI driver behavior cannot be verified by focused local doctest runs, and stdout report migrations for this file cannot be runtime-confirmed until the compile-time blocker is understood.

## 修正方針

Investigate whether the timeout comes from compiler complexity, generated wasm/runtime behavior, selfhost driver fixture size, or an inappropriate test shape. Fix the root cause without weakening static checks or hiding the timeout.

## 検証

Run tests/stdlib/selfhost_cli_driver.n.md focused with --assert-io under the normal timeout, then run the source-policy contract and issues check.
