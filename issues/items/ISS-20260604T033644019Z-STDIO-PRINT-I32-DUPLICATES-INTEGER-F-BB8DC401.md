---
id: ISS-20260604T033644019Z-STDIO-PRINT-I32-DUPLICATES-INTEGER-F-BB8DC401
title: "stdio print_i32 duplicates integer formatting instead of using shared formatter boundary"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/stdio/print.nepl, stdlib/alloc/string/integer/format.nepl"
---

# ISS-20260604T033644019Z-STDIO-PRINT-I32-DUPLICATES-INTEGER-F-BB8DC401: stdio print_i32 duplicates integer formatting instead of using shared formatter boundary

## 概要

node nodesrc/test_stdlib_stdio_print_i32_boundary.js reports that print_i32 no longer delegates integer formatting to alloc/string/integer/format::from_i32. The function locally performs sign/digit branching. This conflicts with the Zenn zero-cost abstraction and responsibility-splitting guidance: the same formatting algorithm should live behind the shared typed formatter boundary, not be duplicated in stdio.

## 対象

- `stdlib/std/stdio/print.nepl, stdlib/alloc/string/integer/format.nepl`

## 根拠

- 未記入

## 問題

node nodesrc/test_stdlib_stdio_print_i32_boundary.js reports that print_i32 no longer delegates integer formatting to alloc/string/integer/format::from_i32. The function locally performs sign/digit branching. This conflicts with the Zenn zero-cost abstraction and responsibility-splitting guidance: the same formatting algorithm should live behind the shared typed formatter boundary, not be duplicated in stdio.

## 影響

Integer formatting behavior can diverge between stdio and alloc/string, and stdio becomes harder to audit because host side effects are mixed with formatting logic.

## 修正方針

Move print_i32 back to the shared integer formatting boundary, keep stdio responsible only for output effects, and add regular tests that compare print_i32 output with formatter output for zero, positive, negative, and i32 min-like boundary values.

## 検証

Run node nodesrc/test_stdlib_stdio_print_i32_boundary.js, focused stdio doctests, and integer format doctests.
