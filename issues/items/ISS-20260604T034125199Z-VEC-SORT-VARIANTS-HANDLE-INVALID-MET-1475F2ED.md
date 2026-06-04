---
id: ISS-20260604T034125199Z-VEC-SORT-VARIANTS-HANDLE-INVALID-MET-1475F2ED
title: "Vec sort variants handle invalid metadata inconsistently and silently no-op"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/alloc/collections/vec/sort/quick.nepl, stdlib/alloc/collections/vec/sort/heap.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl"
---

# ISS-20260604T034125199Z-VEC-SORT-VARIANTS-HANDLE-INVALID-MET-1475F2ED: Vec sort variants handle invalid metadata inconsistently and silently no-op

## 概要

Subagent audit found quick/heap/selection style sort helpers silently returning unit for invalid views while merge sort has Result-shaped error handling. This conflicts with Zenn guidance to avoid silent no-op except documented best-effort effects and to model unsupported/invalid states explicitly.

## 対象

- `stdlib/alloc/collections/vec/sort/quick.nepl, stdlib/alloc/collections/vec/sort/heap.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found quick/heap/selection style sort helpers silently returning unit for invalid views while merge sort has Result-shaped error handling. This conflicts with Zenn guidance to avoid silent no-op except documented best-effort effects and to model unsupported/invalid states explicitly.

## 影響

Sort callers cannot reliably tell whether a sort succeeded, failed due to invalid metadata, or merely did no work; this also hides owner recovery obligations for future non-Copy sort support.

## 修正方針

Unify Vec sort APIs around Result or owner-returning Result, document invalid metadata behavior, and make all variants report errors consistently rather than unit no-op.

## 検証

Run Vec sort doctests and add regular tests for empty, singleton, duplicate, sorted, reverse, invalid metadata, and non-Copy owner-recovery cases.
