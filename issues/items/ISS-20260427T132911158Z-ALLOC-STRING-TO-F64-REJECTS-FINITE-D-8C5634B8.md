---
id: ISS-20260427T132911158Z-ALLOC-STRING-TO-F64-REJECTS-FINITE-D-8C5634B8
title: "alloc/string to_f64 rejects finite decimal strings after digit scan"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/string.nepl; stdlib/tests/string.n.md"
source: ISS-20260427T132142587Z-ALLOC-STRING-DOC-COMMENTS-STILL-CONT-C037036C
---

# ISS-20260427T132911158Z-ALLOC-STRING-TO-F64-REJECTS-FINITE-D-8C5634B8: alloc/string to_f64 rejects finite decimal strings after digit scan

## 概要

to_f64 uses ok = 2 as the final success state, but inputs that end immediately after integer digits, fractional digits, or exponent digits can leave ok at 1. The final success check then returns Result::Err for ordinary finite decimal strings.

## 対象

- `stdlib/alloc/string.nepl; stdlib/tests/string.n.md`

## 根拠

- `to_f64` starts with `ok = 1`, sets `ok = 2` only when a non-digit transition is seen, and returns Ok only when `eq ok 2`.
- There is no focused stdlib test covering `to_f64 "123"`, `to_f64 "-1.5"`, or exponent input.

## 問題

to_f64 uses ok = 2 as the final success state, but inputs that end immediately after integer digits, fractional digits, or exponent digits can leave ok at 1. The final success check then returns Result::Err for ordinary finite decimal strings.

## 影響

Self-host config, diagnostics, and literal parsing need basic floating-point text parsing. Rejecting ordinary finite decimals makes the stdlib conversion unreliable and hides the missing coverage in current string tests.

## 修正方針

Normalize the parser state so a clean end-of-input after required digits is accepted, while malformed decimal/exponent forms still return Err. Add focused regressions for integer-only, fractional, exponent, signed, and invalid inputs.

## 検証

node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/string-to-f64-parser.json -j 1; node nodesrc/issues.js check; git diff --check
