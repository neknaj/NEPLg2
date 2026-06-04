---
id: ISS-20260604T034124436Z-RAW-MEMORY-ALLOCATOR-AND-COLLECTION--C8833FBA
title: "raw memory allocator and collection mutation APIs are exposed as pure functions"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/core/mem/raw.nepl, stdlib/core/mem/allocator.nepl, stdlib/alloc/collections/vec/storage/api.nepl"
---

# ISS-20260604T034124436Z-RAW-MEMORY-ALLOCATOR-AND-COLLECTION--C8833FBA: raw memory allocator and collection mutation APIs are exposed as pure functions

## 概要

Subagent audit found mem_grow, raw store/load style helpers, allocator operations, and Vec allocation/mutation APIs exposed through %fn even though they mutate memory or ownership state. This conflicts with plan.md pure/impure function separation and the Zenn policy that side effects must be surfaced by the type system.

## 対象

- `stdlib/core/mem/raw.nepl, stdlib/core/mem/allocator.nepl, stdlib/alloc/collections/vec/storage/api.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found mem_grow, raw store/load style helpers, allocator operations, and Vec allocation/mutation APIs exposed through %fn even though they mutate memory or ownership state. This conflicts with plan.md pure/impure function separation and the Zenn policy that side effects must be surfaced by the type system.

## 影響

Pure stdlib functions can appear to call allocation or raw memory mutation without an impure boundary, weakening static effect checks and hiding platform/resource side effects in core and alloc APIs.

## 修正方針

Move raw memory, allocator, and ownership-changing collection operations to impure signatures or compiler-known proof boundaries, keep pure wrappers only for read-only value computations, and document the exact effect contract.

## 検証

Add compile-fail tests where a pure function calls allocation/store/grow APIs, plus focused success/failure tests for Vec new/push/free once cfg-test-style regular tests are available.
