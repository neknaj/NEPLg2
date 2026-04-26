---
id: ISS-20260426T213056969Z-CI-WASIX-DOCTESTS-RUN-WITHOUT-A-WASM-B1BD31BA
title: "CI wasix doctests run without a wasmer executable"
area: cli
status: open
resolved: false
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-26
target: ".github/workflows/ci.yml, nodesrc/run_test.js, tests/stdlib/features_tui.n.md"
---

# ISS-20260426T213056969Z-CI-WASIX-DOCTESTS-RUN-WITHOUT-A-WASM-B1BD31BA: CI wasix doctests run without a wasmer executable

## 概要

GitHub Actions run 24967172989 fails tests/stdlib/features_tui.n.md doctest#1-#4 in nmd-doctest, wasi-test, and llvm-dual-tests with Error: spawn wasmer ENOENT. Existing Wasmer issues cover option compatibility and tty imports, but not the missing executable in CI.

## 対象

- `.github/workflows/ci.yml, nodesrc/run_test.js, tests/stdlib/features_tui.n.md`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 fails tests/stdlib/features_tui.n.md doctest#1-#4 in nmd-doctest, wasi-test, and llvm-dual-tests with Error: spawn wasmer ENOENT. Existing Wasmer issues cover option compatibility and tty imports, but not the missing executable in CI.

## 影響

All #target wasix doctests are red before compiler/runtime checks run, and TUI regressions are masked by CI environment setup.

## 修正方針

Install a pinned Wasmer in every workflow job that can run #target wasix tests, or make the test runner detect missing wasmer and route supported WASIX cases through the Node fallback with an explicit diagnostic.

## 検証

Run GitHub Actions nmd-doctest and wasi-test and confirm features_tui doctest#1-#4 no longer fail with spawn wasmer ENOENT.
