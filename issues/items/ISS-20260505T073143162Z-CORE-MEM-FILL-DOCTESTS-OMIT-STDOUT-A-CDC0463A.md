---
id: ISS-20260505T073143162Z-CORE-MEM-FILL-DOCTESTS-OMIT-STDOUT-A-CDC0463A
title: "core mem fill doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/mem.nepl
---

# ISS-20260505T073143162Z-CORE-MEM-FILL-DOCTESTS-OMIT-STDOUT-A-CDC0463A: core mem fill doctests omit stdout assertion reports

## 概要

The memset_u8 and fill_i32 doc-comment doctests use std/test checks after low-level raw memory operations but return checks_exit_code checks without printing deterministic assertion reports.

## 対象

- `stdlib/core/mem.nepl`

## 根拠

- `stdlib/core/mem.nepl` の `memset_u8` / `fill_i32` doc-comment doctest は `std/test` checks で 2 件ずつ load/store assertion を集約していた。
- 修正前は `checks_exit_code checks` だけを返し、低レベルメモリ helper の観測結果を stdout report fixture として固定していなかった。

## 問題

The memset_u8 and fill_i32 doc-comment doctests use std/test checks after low-level raw memory operations but return checks_exit_code checks without printing deterministic assertion reports.

## 影響

Memory helper regressions only expose success/failure through exit status, and the checked load/store observations are not fixed as stdout fixtures for runner parity.

## 修正方針

Add exit_code metadata and checks_print_report stdout fixtures while preserving the existing alloc/write/read/dealloc order and memory-safety intent.

## 対応結果

- `memset_u8` / `fill_i32` doctest に `exit_code: 0` と `Checked [ok,ok]` stdout fixture を追加した。
- 既存の `alloc_raw`、書き込み、読み取り、`dealloc_raw` の順序は維持し、メモリ操作の意味は変えていない。
- `checks_print_report` は `dealloc_raw` 後に呼び、既存の cleanup を遅らせない形にした。
- full file run では既存の `doctest#3` が `resource.cell.uninit` で失敗したため、`ISS-20260505T073434026Z-CORE-MEM-ALLOCATOR-METADATA-DOCTEST--3D5EEF97` として分離した。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 5 --dist web/dist`: passed, stdout=`Checked [ok,ok]`
- `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 6 --dist web/dist`: passed, stdout=`Checked [ok,ok]`
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/core/mem.nepl --no-tree -o tmp/core-mem-fill-report-agent1.json -j 1 --dist web/dist`: total=6, passed=5, failed=1。失敗は未変更の `doctest#3` で、別 issue に分離済み。
