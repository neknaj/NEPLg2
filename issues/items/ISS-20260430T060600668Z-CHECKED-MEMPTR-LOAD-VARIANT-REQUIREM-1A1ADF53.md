---
id: ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53
title: "Checked MemPtr load variant requirements lack impossible-branch refinement"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/initialized_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53: Checked MemPtr load variant requirements lack impossible-branch refinement

## 概要

load_i32(MemPtr<i32>) correctly requires the pointee cell to be initialized when the call result is Option::Some, but the checker applies the Some requirement to every syntactic Some arm even when the wrapper guard is known to return Option::None for a statically invalid pointer such as mem_ptr_wrap 0.

## 対象

- `nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/initialized_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- 未記入

## 問題

load_i32(MemPtr<i32>) correctly requires the pointee cell to be initialized when the call result is Option::Some, but the checker applies the Some requirement to every syntactic Some arm even when the wrapper guard is known to return Option::None for a statically invalid pointer such as mem_ptr_wrap 0.

## 影響

Safe invalid-pointer handling examples that expect Option::None are rejected with resource.cell.uninit. Disabling the Some requirement would be unsound, so the missing piece is path refinement for wrapper guard facts rather than weakening the raw load precondition.

## 修正方針

Carry branch condition facts from checked MemPtr wrappers into the variant summary, or add a Resource IR representation of impossible returned variants for calls whose guard condition can be proven. Apply variant requirements only to reachable match arms while keeping unknown pointers conservative.

## 検証

Add Resource IR regressions for mem_ptr_wrap 0 -> load_i32 -> Option::None passing, and an unknown/uninitialized MemPtr -> Option::Some branch still reporting RawMemoryLoadCell.
