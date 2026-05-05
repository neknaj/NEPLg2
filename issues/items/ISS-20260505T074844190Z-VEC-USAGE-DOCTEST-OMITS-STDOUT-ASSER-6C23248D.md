---
id: ISS-20260505T074844190Z-VEC-USAGE-DOCTEST-OMITS-STDOUT-ASSER-6C23248D
title: "vec usage doctest omits stdout assertion report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/alloc/collections/vec.nepl
---

# ISS-20260505T074844190Z-VEC-USAGE-DOCTEST-OMITS-STDOUT-ASSER-6C23248D: vec usage doctest omits stdout assertion report

## 概要

The primary Vec usage doc-comment doctest builds std/test checks for len and get behavior but returns checks_exit_code checks2 without printing a deterministic assertion report.

## 対象

- `stdlib/alloc/collections/vec.nepl`

## 根拠

- `stdlib/alloc/collections/vec.nepl::doctest#1` は `std/test` checks で `len` と `get` の基本挙動を 3 件確認していた。
- 修正前は `checks_exit_code checks2` だけを返し、stdout report を fixture として固定していなかった。
- 変更前の focused run は `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1 --dist web/dist` で pass していたが、stdout は空だった。

## 問題

The primary Vec usage doc-comment doctest builds std/test checks for len and get behavior but returns checks_exit_code checks2 without printing a deterministic assertion report.

## 影響

Vec example regressions can be observed only through exit success, while the basic collection assertions are not pinned as stdout for Rust/selfhost runner parity.

## 修正方針

Add exit_code metadata and a checks_print_report stdout fixture to the focused usage doctest, preserving allocation and free order.

## 対応結果

- doctest metadata に `exit_code: 0` と `Checked [ok,ok,ok]` stdout fixture を追加した。
- `free a2` / `free b2` の後に `checks_print_report` を呼ぶ形にし、既存の owner cleanup order は維持した。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1 --dist web/dist`: passed, stdout=`Checked [ok,ok,ok]`
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
