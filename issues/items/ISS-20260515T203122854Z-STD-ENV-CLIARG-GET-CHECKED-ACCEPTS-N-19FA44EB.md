---
id: ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB
title: "std/env cliarg_get_checked accepts negative index before argv slot access"
area: stdlib
status: open
resolved: false
priority: P0
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/env/cliarg/raw.nepl; stdlib/tests/cliarg.n.md; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB: std/env cliarg_get_checked accepts negative index before argv slot access

## 概要

cliarg_get_checked only rejects idx >= argc and buf_size <= 0 before computing arg_slot_raw = argv_raw + idx * 4, so a negative idx can address before the argv pointer array inside the raw argv boundary.

## 対象

- `stdlib/std/env/cliarg/raw.nepl; stdlib/tests/cliarg.n.md; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

cliarg_get_checked only rejects idx >= argc and buf_size <= 0 before computing arg_slot_raw = argv_raw + idx * 4, so a negative idx can address before the argv pointer array inside the raw argv boundary.

## 影響

A safe public cliarg_get negative index currently reaches raw pointer slot arithmetic, which violates the memory-safety requirement even if host args_get later fails or returns no useful string.

## 修正方針

Reject idx < 0 before allocating argv scratch or computing arg_slot_raw, add a regression doctest for cliarg_get negative index, and add source policy coverage for the lower-bound check.

## 検証

Run cliarg source policy and focused cliarg doctests.
