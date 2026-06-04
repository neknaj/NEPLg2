---
id: ISS-20260604T033642997Z-STDLIB-DOCUMENTATION-AND-DOCTEST-REP-E892BD11
title: "stdlib documentation and doctest report contracts still have ret-only and stale baseline gaps"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/core/gui, stdlib/alloc/gui, stdlib/std/gui, stdlib/platforms/gui, tests/stdlib/gui_*.n.md, stdlib/alloc/string/integer/parse.nepl"
---

# ISS-20260604T033642997Z-STDLIB-DOCUMENTATION-AND-DOCTEST-REP-E892BD11: stdlib documentation and doctest report contracts still have ret-only and stale baseline gaps

## 概要

Current source policy reports stdlib documentation contract drift and alloc string integer parse report mismatch. A broad rg pass also finds many GUI module doctests and tests/stdlib/gui_*.n.md entries using ret-only metadata or Checked-style output instead of canonical TestReport stdout. Zenn requires doc comments to state contract/current implementation, enum return cases, complexity, simple and typical doctests, plus separate detailed tests.

## 対象

- `stdlib/core/gui, stdlib/alloc/gui, stdlib/std/gui, stdlib/platforms/gui, tests/stdlib/gui_*.n.md, stdlib/alloc/string/integer/parse.nepl`

## 根拠

- 未記入

## 問題

Current source policy reports stdlib documentation contract drift and alloc string integer parse report mismatch. A broad rg pass also finds many GUI module doctests and tests/stdlib/gui_*.n.md entries using ret-only metadata or Checked-style output instead of canonical TestReport stdout. Zenn requires doc comments to state contract/current implementation, enum return cases, complexity, simple and typical doctests, plus separate detailed tests.

## 影響

The audit cannot distinguish a documented contract from a currently observed implementation, and doctests can pass without exposing the behavior that changed. GUI/TUI work is especially exposed because many docs are new and still use minimal ret-only checks.

## 修正方針

Replace ret-only doctests with canonical std/test reports where behavior is observable, update documentation contract baselines only after real gaps are triaged, and split doc examples from future cfg-test-style regular tests for layout/event/render edge cases.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, node nodesrc/test_alloc_string_doc_report_contract.js, focused GUI doctests, and future cfg-test regular suites for GUI layout/event/render behavior.
