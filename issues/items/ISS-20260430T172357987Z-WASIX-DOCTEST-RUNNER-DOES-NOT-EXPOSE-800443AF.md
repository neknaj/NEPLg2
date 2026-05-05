---
id: ISS-20260430T172357987Z-WASIX-DOCTEST-RUNNER-DOES-NOT-EXPOSE-800443AF
title: "WASIX doctest runner does not expose return or exit-code result"
area: cli
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-05-05
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

## 2026-05-05 対応結果

- `run_test.js` に `decodeRunExitCode` を追加し、Wasmer 経路の `exitCode` と Node WASI fallback 経路の `returnValue` を同じ doctest `exit_code` として扱うようにした。
- Wasmer の process exit code が nonzero の場合でも、doctest が `exit_code:` を期待しているなら runtime phase だけで即 fail せず、`run_doctest.js` / `tests.js` 側の exit code expectation で一致・不一致を判定するようにした。
- `ret:` は WASIX では portable な契約にしない方針を `nodesrc/README.n.md` に明記した。WASIX の成功/失敗判定は `exit_code:` を使う。
- `nodesrc/test_doctest_exit_code_metadata.js` に `#target wasix` の focused regression を追加し、matching `exit_code: 7` が pass、mismatch が `exit code mismatch` になることを固定した。
- `nodesrc/test_run_test_wasix_missing_wasmer_fallback.js` に Wasmer process exit / Node fallback return の exit code decode と runtime phase 判定の unit regression を追加した。

## 2026-05-05 検証

- `node nodesrc/test_run_test_wasix_missing_wasmer_fallback.js`: passed
- `node nodesrc/test_doctest_exit_code_metadata.js`: passed
