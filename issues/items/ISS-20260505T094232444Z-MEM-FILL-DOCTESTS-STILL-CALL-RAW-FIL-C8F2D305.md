---
id: ISS-20260505T094232444Z-MEM-FILL-DOCTESTS-STILL-CALL-RAW-FIL-C8F2D305
title: "mem_fill doctests still call raw fill helpers from pure main"
area: TEST
status: open
resolved: false
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/mem_fill.n.md
---

# ISS-20260505T094232444Z-MEM-FILL-DOCTESTS-STILL-CALL-RAW-FIL-C8F2D305: mem_fill doctests still call raw fill helpers from pure main

## 概要

After resource.raw.unsafe_memory_boundary became an enforced diagnostic, tests/stdlib/mem_fill.n.md still declares raw fill positive doctest main functions as pure while calling memset_u8, fill_i32, and fill_u8.

## 対象

- `tests/stdlib/mem_fill.n.md`

## 根拠

- 未記入

## 問題

After resource.raw.unsafe_memory_boundary became an enforced diagnostic, tests/stdlib/mem_fill.n.md still declares raw fill positive doctest main functions as pure while calling memset_u8, fill_i32, and fill_u8.

## 影響

Focused mem_fill doctests fail at the effect boundary before they can verify fill behavior. This is a test fixture issue separate from Resource IR fill range correctness.

## 修正方針

Mark only the positive raw fill doctest main functions as impure, keeping future compile_fail boundary tests pure when added.

## 検証

Run node nodesrc/tests.js -i tests/stdlib/mem_fill.n.md --no-tree after trunk build.
