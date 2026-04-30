---
id: ISS-20260430T020255446Z-GETTING-STARTED-DOCTESTS-RELY-ON-EXI-8902362D
title: "Getting started doctests rely on exit code without stdout reports"
area: tutorials
status: fixed
resolved: true
priority: P2
type: test
created: 2026-04-30
updated: 2026-04-30
target: "tutorials/getting_started/*.n.md"
---

# ISS-20260430T020255446Z-GETTING-STARTED-DOCTESTS-RELY-ON-EXI-8902362D: Getting started doctests rely on exit code without stdout reports

## 概要

Most getting_started tutorial doctests aggregate std/test checks but only assert ret: 0, so generated tutorial examples do not show assertion results in stdout.

## 対象

- `tutorials/getting_started/*.n.md`

## 根拠

- `tutorials/getting_started/02_test_harness.n.md` 以外の多くの doctest が `checks_exit_code checks` だけを返し、stdout に検査結果を表示していなかった。
- `std/test` の現行 API は `assert` / `assert_*` で構造化 assertion を作り、`checks_print_report` で stdout に安定した check report を出す設計になっている。

## 問題

Most getting_started tutorial doctests aggregate std/test checks but only assert ret: 0, so generated tutorial examples do not show assertion results in stdout.

## 影響

Readers and CI artifacts cannot see which tutorial assertions were exercised, and a tutorial can appear as an exit-code-only smoke test rather than executable documentation with visible results.

## 修正方針

Update tutorial doctests that use std/test to print their check report before returning checks_exit_code, and add stdout expectations for the report output.

## 検証

Run tutorial doctests and example doctests through nodesrc/tests.js with JSON output, then inspect failures and totals.

確認済み:

- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-stdout-tests.json -j 4` (`total=24`, `passed=24`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 4` は `total=12`, `passed=7`, `failed=5`。失敗は `examples/bf.nepl`, `examples/rpn.nepl`, `examples/rpn_legacy.nepl` が `stk::push_ref` / `stk::pop_ref` を参照している別問題として分離する。
