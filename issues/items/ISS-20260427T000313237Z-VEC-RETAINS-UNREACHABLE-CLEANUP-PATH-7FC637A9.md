---
id: ISS-20260427T000313237Z-VEC-RETAINS-UNREACHABLE-CLEANUP-PATH-7FC637A9
title: "Vec retains unreachable cleanup paths in owned buffer internals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/vec_collections.n.md"
---

# ISS-20260427T000313237Z-VEC-RETAINS-UNREACHABLE-CLEANUP-PATH-7FC637A9: Vec retains unreachable cleanup paths in owned buffer internals

## 概要

Vec.free and scratch-buffer sort cleanup still match dealloc_ptr errors to unreachable instead of using explicit owner-invariant cleanup.

## 対象

- `stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/vec_collections.n.md`

## 根拠

- 未記入

## 問題

Vec.free and scratch-buffer sort cleanup still match dealloc_ptr errors to unreachable instead of using explicit owner-invariant cleanup.

## 影響

Vec is the central self-host container; unreachable cleanup paths make allocator regressions hard to diagnose and keep normal internals dependent on impossible branches.

## 修正方針

Use dealloc_raw for owned Vec/scratch buffers where ownership is established, keep external allocation failures as Result, and add source and behavior regressions.

## 検証

Run Vec doctests, vec sort doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
