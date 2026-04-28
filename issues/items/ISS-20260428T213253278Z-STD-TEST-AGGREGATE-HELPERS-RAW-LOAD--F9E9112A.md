---
id: ISS-20260428T213253278Z-STD-TEST-AGGREGATE-HELPERS-RAW-LOAD--F9E9112A
title: "std/test aggregate helpers raw-load Vec backing store under RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: stdlib/std/test.nepl
---

# ISS-20260428T213253278Z-STD-TEST-AGGREGATE-HELPERS-RAW-LOAD--F9E9112A: std/test aggregate helpers raw-load Vec backing store under RawMemoryLoadCell gate

## 概要

After remote main enabled the RawMemoryLoadCell gate, std/test doctests fail because checks_has_err_loop, checks_summary_loop, and checks_print_human_loop read Vec<Result<(),str>> elements through raw data pointers with load<Result<(),str>>. The resource checker cannot prove those temporary raw cells are initialized.

## 対象

- `stdlib/std/test.nepl`

## 根拠

- 未記入

## 問題

After remote main enabled the RawMemoryLoadCell gate, std/test doctests fail because checks_has_err_loop, checks_summary_loop, and checks_print_human_loop read Vec<Result<(),str>> elements through raw data pointers with load<Result<(),str>>. The resource checker cannot prove those temporary raw cells are initialized.

## 影響

All std/test doctests fail on current main, and any doctest importing std/test can fail before testing its own module. This blocks stdlib and self-host regression tests.

## 修正方針

Replace raw backing-store scans in std/test with ownership-safe Vec accessors or introduce a safe iteration helper that preserves initialization information, then add a focused regression for checks_exit_code/checks_summary.

## 検証

Run stdlib/std/test.nepl doctests, the blocked self-host prelude doctest, issue index/check, and git diff --check.
