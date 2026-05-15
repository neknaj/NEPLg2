---
id: ISS-20260515T130627053Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-3CDEFF1A
title: "Stdlib documentation contract declaration doctest baseline regressed to 1038"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260515T130627053Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-3CDEFF1A: Stdlib documentation contract declaration doctest baseline regressed to 1038

## 概要

node nodesrc/test_stdlib_documentation_contract.js now reports declaration doctest gaps increased to 1038 while the frozen baseline is 1032. This reopens the documentation contract signal after previous fixes brought the count back to 1032.

## 対象

- `stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- 未記入

## 問題

node nodesrc/test_stdlib_documentation_contract.js now reports declaration doctest gaps increased to 1038 while the frozen baseline is 1032. This reopens the documentation contract signal after previous fixes brought the count back to 1032.

## 影響

Source policy warn-only hides an executable documentation coverage regression. New stdlib APIs may lack typical-use doctests despite the project rule that docs and doctests are part of the API contract.

## 修正方針

Audit the six-gap regression, add meaningful declaration doctests for the changed public APIs instead of raising the baseline, and keep the policy baseline at or below 1032.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, focused doctests for updated files, source policy warn-only, issues check, and diff whitespace check.
