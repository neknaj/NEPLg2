---
id: ISS-20260427T031251079Z-LIST-MAP-AND-FILTER-LEAK-PARTIAL-RES-9B715B6A
title: "List map and filter leak partial results when final cons allocation fails"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md"
---

# ISS-20260427T031251079Z-LIST-MAP-AND-FILTER-LEAK-PARTIAL-RES-9B715B6A: List map and filter leak partial results when final cons allocation fails

## 概要

list_map_impl and list_filter_impl recursively allocate the tail first and then call cons for the current head. If that final cons allocation fails, the already-built mapped or filtered tail is dropped from the Result path without cleanup.

## 対象

- `stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md`

## 根拠

- 未記入

## 問題

list_map_impl and list_filter_impl recursively allocate the tail first and then call cons for the current head. If that final cons allocation fails, the already-built mapped or filtered tail is dropped from the Result path without cleanup.

## 影響

Allocation pressure in self-host list transformations can leak nodes and leave callers with only an Err, making repeated parser/helper transforms progressively exhaust memory.

## 修正方針

When cons fails after a tail list has been built, free the partial tail before returning Err, and add a source regression covering the cleanup path.

## 検証

Run list doctests, focused list collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
