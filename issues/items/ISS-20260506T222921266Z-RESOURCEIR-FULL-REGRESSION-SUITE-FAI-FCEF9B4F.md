---
id: ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F
title: "ResourceIR full regression suite fails on origin/main baseline"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F: ResourceIR full regression suite fails on origin/main baseline

## 概要

On clean origin/main 3ba24e72, cargo test -p nepl-core --test resource_ir -- --nocapture fails 10 existing ResourceIR tests: branch borrow merge, returned raw address helper/function/aggregate-field alias, borrowed region ptr alias, literal zero offset raw helper, unknown-offset region dealloc, lowering skeleton dump, double dealloc, and returned aggregate raw cell owner field alias. The current owner-summary fix branch shows the same 10 baseline failures while its newly added tests pass.

## 対象

- `nepl-core/src/resource, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

On clean origin/main 3ba24e72, cargo test -p nepl-core --test resource_ir -- --nocapture fails 10 existing ResourceIR tests: branch borrow merge, returned raw address helper/function/aggregate-field alias, borrowed region ptr alias, literal zero offset raw helper, unknown-offset region dealloc, lowering skeleton dump, double dealloc, and returned aggregate raw cell owner field alias. The current owner-summary fix branch shows the same 10 baseline failures while its newly added tests pass.

## 影響

A non-clean ResourceIR baseline weakens static-check verification for type and memory safety. New owner/raw-address changes cannot rely on the full ResourceIR suite as a regression signal, and real regressions can be hidden among known failures.

## 修正方針

Triage the 10 failures by root cause and fix them without expected-fail masking. Preserve explicit raw-address proof requirements, restore initialized-cell/owner summaries for legitimate raw pointer flows, and repair borrow merge reporting so memory and borrow safety diagnostics remain authoritative.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture must pass on clean origin/main; keep focused regressions for each root cause and run source-policy resource checker guards.
