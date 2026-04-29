---
id: ISS-20260429T125051782Z-STRING-FROM-MEM-UNCHECKED-RESULT-LEA-DEC43497
title: "string_from_mem_unchecked_result leaks output region owner under Resource IR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, tests/stdlib/stdin.n.md, tests/stdlib/streamio.n.md"
---

# ISS-20260429T125051782Z-STRING-FROM-MEM-UNCHECKED-RESULT-LEA-DEC43497: string_from_mem_unchecked_result leaks output region owner under Resource IR

## 概要

After origin/main 78f310e, stdin and streamio focused runs report string_from_mem_unchecked_result leaking the allocated output region owner. This appears separately from the existing concat_result owner issue and blocks scanner token/string conversion validation.

## 対象

- `stdlib/alloc/string.nepl, tests/stdlib/stdin.n.md, tests/stdlib/streamio.n.md`

## 根拠

- 未記入

## 問題

After origin/main 78f310e, stdin and streamio focused runs report string_from_mem_unchecked_result leaking the allocated output region owner. This appears separately from the existing concat_result owner issue and blocks scanner token/string conversion validation.

## 影響

Any stdlib or self-host path that copies bytes into a new str can fail the memory-safety gate even when the caller's byte access has been made ResourceIR-safe. This blocks stdin read_line, stream scanner tokens, and string-heavy self-host components.

## 修正方針

Review the string constructor ownership contract. Make the allocated region owner move into the returned str on every Ok path, and make every Err path free or avoid allocating that region. Keep UTF-8 checked and unchecked constructors sharing a single owner-safe construction boundary.

## 検証

Run focused alloc/string constructor doctests, tests/stdlib/stdin.n.md, and tests/stdlib/streamio.n.md after the constructor owner transfer is fixed.
