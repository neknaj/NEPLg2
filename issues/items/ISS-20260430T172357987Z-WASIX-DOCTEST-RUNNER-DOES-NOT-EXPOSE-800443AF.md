---
id: ISS-20260430T172357987Z-WASIX-DOCTEST-RUNNER-DOES-NOT-EXPOSE-800443AF
title: "WASIX doctest runner does not expose return or exit-code result"
area: cli
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nodesrc/run_test.js, nodesrc/run_doctest.js, nodesrc/tests.js"
---

# ISS-20260430T172357987Z-WASIX-DOCTEST-RUNNER-DOES-NOT-EXPOSE-800443AF: WASIX doctest runner does not expose return or exit-code result

## 概要

Focused features_tui WASIX doctest verification showed that `run_doctest` reports `return value mismatch ... actual: null` for `ret: 0` and `exit code result missing` for `exit_code: 0`, even though the target executes stdout-capable WASIX cases through the Node WASI fallback.

## 対象

- `nodesrc/run_test.js, nodesrc/run_doctest.js, nodesrc/tests.js`

## 根拠

- 未記入

## 問題

Focused features_tui WASIX doctest verification showed that `run_doctest` reports `return value mismatch ... actual: null` for `ret: 0` and `exit code result missing` for `exit_code: 0`, even though the target executes stdout-capable WASIX cases through the Node WASI fallback.

## 影響

WASIX assertion suites cannot use the same stdout plus exit_code contract as std/WASI doctests, leaving process success/failure metadata inconsistent across Rust and future self-host runners.

## 修正方針

Make the WASIX runner surface an explicit exit_code when the runtime finishes successfully or fails, and document whether NEPL main return values are meaningful for WASIX targets. Then update WASIX std/test fixtures to use exit_code rather than ret where appropriate.

## 検証

Add a runner regression with a tiny #target wasix program that returns 0 and one that returns nonzero or traps, and confirm run_doctest/tests.js can distinguish exit_code from ret.
