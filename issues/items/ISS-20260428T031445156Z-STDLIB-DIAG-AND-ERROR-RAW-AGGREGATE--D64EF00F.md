---
id: ISS-20260428T031445156Z-STDLIB-DIAG-AND-ERROR-RAW-AGGREGATE--D64EF00F
title: "stdlib diag and error raw aggregate detours fail under strict move checking"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/alloc/diag/error.nepl, stdlib/tests/diag.n.md, stdlib/tests/error.n.md"
---

# ISS-20260428T031445156Z-STDLIB-DIAG-AND-ERROR-RAW-AGGREGATE--D64EF00F: stdlib diag and error raw aggregate detours fail under strict move checking

## 概要

Latest strict move checking rejects diag/error helpers that store Diag or aggregate Result-like values in raw memory and repeatedly load fields from the same non-Copy raw place. stdlib/tests/diag.n.md and stdlib/tests/error.n.md now fail with D3100 moved raw memory / deallocating raw memory containing non-Copy values.

## 対象

- `stdlib/alloc/diag/error.nepl, stdlib/tests/diag.n.md, stdlib/tests/error.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests --no-tree -o tmp/stdlib-tests-static-followup-20260428.json -j 1` で `stdlib/tests/diag.n.md::doctest#1/#2` と `stdlib/tests/error.n.md::doctest#1/#2/#3` が D3100 になった。
- 主な失敗は `use of moved raw memory place: d_mem`、`deallocating raw memory place containing non-Copy value: notes_mem`、`deallocating raw memory place containing non-Copy value: items_mem`、`use of moved raw memory place: r0_mem`。
- `Diag` や diagnostic container を raw memory に置いて field を繰り返し `load` する実装が、最新の strict move checking で non-Copy aggregate の二重 move / live payload dealloc として表面化している。

## 問題

Latest strict move checking rejects diag/error helpers that store Diag or aggregate Result-like values in raw memory and repeatedly load fields from the same non-Copy raw place. stdlib/tests/diag.n.md and stdlib/tests/error.n.md now fail with D3100 moved raw memory / deallocating raw memory containing non-Copy values.

## 影響

stdlib diagnostic helpers are part of error reporting and self-host infrastructure. Leaving them as raw aggregate detours blocks clean stdlib verification and encourages weakening D3100 instead of removing unsafe aggregate decomposition patterns.

## 修正方針

Replace raw aggregate detours with borrowed field projection or owned decomposition that does not repeatedly load non-Copy aggregates from raw memory. Keep the move checker strict and add focused diag/error doctest regressions.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md --no-tree -o tmp/stdlib-diag-error-after-fix.json -j 1 and node nodesrc/issues.js check.
