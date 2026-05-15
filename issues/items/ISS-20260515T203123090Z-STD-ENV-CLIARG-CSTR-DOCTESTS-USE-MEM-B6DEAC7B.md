---
id: ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B
title: "std/env cliarg cstr doctests use mem_ptr_add outside raw boundary"
area: stdlib
status: open
resolved: false
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/env/cliarg/cstr.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260515T203123090Z-STD-ENV-CLIARG-CSTR-DOCTESTS-USE-MEM-B6DEAC7B: std/env cliarg cstr doctests use mem_ptr_add outside raw boundary

## 概要

std/env/cliarg/cstr.nepl doctests allocate RegionToken then write test bytes through mem_ptr_add/store_u8 in ordinary doctest source, which is rejected by Resource IR as resource.raw.memory_outside_boundary.

## 対象

- `stdlib/std/env/cliarg/cstr.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

std/env/cliarg/cstr.nepl doctests allocate RegionToken then write test bytes through mem_ptr_add/store_u8 in ordinary doctest source, which is rejected by Resource IR as resource.raw.memory_outside_boundary.

## 影響

The cstr helper documentation examples are stale under the current Stage 6 raw boundary model and cannot serve as regression tests without weakening static checks.

## 修正方針

Rewrite cstr doctests to use safe public construction helpers or a compile_fail raw-boundary example, without granting raw memory authority to ordinary doctest code.

## 検証

Run cstr doctests and cliarg source policy.
