---
id: ISS-20260430T024859462Z-FENWICK-SUM-OBSERVER-APIS-CONSUME-OW-2BB59F00
title: "Fenwick sum observer APIs consume owners by value instead of borrowing"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/fenwick.nepl
---

# ISS-20260430T024859462Z-FENWICK-SUM-OBSERVER-APIS-CONSUME-OW-2BB59F00: Fenwick sum observer APIs consume owners by value instead of borrowing

## 概要

Fenwick len/sum_prefix/sum_range take Fenwick by value although they only read the internal tree. A range query moves the tree owner and prevents later cleanup, which is the same root ownership-design defect fixed for BitSet.

## 対象

- `stdlib/alloc/collections/fenwick.nepl`

## 根拠

- `stdlib/alloc/collections/fenwick.nepl` に `fn len <(Fenwick)->i32>`、`fn sum_prefix <(Fenwick,i32)*>Result<i32, Diag>>`、`fn sum_range <(Fenwick,i32,i32)*>Result<i32, Diag>>` が残っている。
- BitSet の owner-consuming observer 修正中に raw-array collection を確認し、internal tree を読むだけの Fenwick query API も値 receiver のままだと判明した。

## 問題

Fenwick len/sum_prefix/sum_range take Fenwick by value although they only read the internal tree. A range query moves the tree owner and prevents later cleanup, which is the same root ownership-design defect fixed for BitSet.

## 影響

Self-host numeric analysis code cannot safely run multiple Fenwick queries or free the tree after a query without bypassing the public APIs. This undermines mandatory memory-safety checking.

## 修正方針

Change Fenwick observer/query APIs to take &Fenwick, read Copy fields from borrowed references, update doctests/tests to query through borrowed receivers, and keep add as the owner-consuming update API.

## 検証

Add tests that perform several borrowed sum queries on one Fenwick value and then free it without resource diagnostics.
