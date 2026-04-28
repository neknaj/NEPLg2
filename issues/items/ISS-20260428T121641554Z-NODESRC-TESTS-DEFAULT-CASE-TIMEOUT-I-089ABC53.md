---
id: ISS-20260428T121641554Z-NODESRC-TESTS-DEFAULT-CASE-TIMEOUT-I-089ABC53
title: "nodesrc tests default case timeout is too short for selfhost parser doctests"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nodesrc/tests.js, doc/testing.md"
---

# ISS-20260428T121641554Z-NODESRC-TESTS-DEFAULT-CASE-TIMEOUT-I-089ABC53: nodesrc tests default case timeout is too short for selfhost parser doctests

## 概要

The new selfhost parser doctest compiles and runs successfully with NEPL_TEST_CASE_TIMEOUT_MS=60000, but the default 20000ms case timeout expires before completion. This makes valid selfhost parser regression tests fail in normal CI-style invocations.

## 対象

- `nodesrc/tests.js, doc/testing.md`

## 根拠

- 未記入

## 問題

The new selfhost parser doctest compiles and runs successfully with NEPL_TEST_CASE_TIMEOUT_MS=60000, but the default 20000ms case timeout expires before completion. This makes valid selfhost parser regression tests fail in normal CI-style invocations.

## 影響

Selfhost S1 parser coverage cannot be added reliably, and agents must remember an environment override that is easy to omit. Timeout failures also look like runtime hangs even when the compiler/test is only slower than the old default.

## 修正方針

Raise or document the default case timeout used by nodesrc/tests.js so selfhost stdlib/parser doctests pass without ad-hoc environment overrides, while keeping the environment variable available for stricter local runs.

## 検証

node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree; node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree; node nodesrc/issues.js check

## 2026-04-28 修正

- `nodesrc/tests.js` に `DEFAULT_TEST_CASE_TIMEOUT_MS = 60000` を追加し、thread pool runner と subprocess runner の既定 case timeout を 60 秒へ統一した。
- `NEPL_TEST_CASE_TIMEOUT_MS` は従来通り優先されるため、ローカルで 20 秒などの厳しい timeout を明示する運用は維持できる。
- `doc/testing.md` に既定 timeout と override 方法を追記した。
