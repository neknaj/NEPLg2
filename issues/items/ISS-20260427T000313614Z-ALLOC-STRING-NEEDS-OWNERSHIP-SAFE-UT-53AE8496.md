---
id: ISS-20260427T000313614Z-ALLOC-STRING-NEEDS-OWNERSHIP-SAFE-UT-53AE8496
title: "alloc/string needs ownership-safe UTF-8 reimplementation"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/string.nepl, stdlib/tests/string.n.md, tests/stdlib/string.n.md, stdlib/std/fs.nepl"
---

# ISS-20260427T000313614Z-ALLOC-STRING-NEEDS-OWNERSHIP-SAFE-UT-53AE8496: alloc/string needs ownership-safe UTF-8 reimplementation

## 概要

alloc/string still mixes raw region layout, unwrap_ok/unwrap/unreachable paths, byte-based constructors, and older compiler-workaround temporaries, so str invariants and allocation failures are not represented consistently.

## 対象

- `stdlib/alloc/string.nepl, stdlib/tests/string.n.md, tests/stdlib/string.n.md, stdlib/std/fs.nepl`

## 根拠

- 未記入

## 問題

alloc/string still mixes raw region layout, unwrap_ok/unwrap/unreachable paths, byte-based constructors, and older compiler-workaround temporaries, so str invariants and allocation failures are not represented consistently.

## 影響

Self-host lexer, parser, diagnostics, module paths, HTML/NM generation, and file loading all depend on trustworthy UTF-8 strings and predictable Result-returning string operations.

## 修正方針

Rework the string core around explicit owned raw operations and checked UTF-8 construction, remove compiler-workaround intermediate variables where the fixed compiler allows direct expressions, and expand regression coverage for allocation, slicing, search, formatting, and invalid UTF-8 boundaries.

## 検証

Run string doctests, stdlib string fixtures, fs string conversion fixtures, source guard for unsafe helpers, full stdlib suite, and nodesrc/issues.js check.
