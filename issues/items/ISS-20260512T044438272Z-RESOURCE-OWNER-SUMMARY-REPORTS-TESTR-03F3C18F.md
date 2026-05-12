---
id: ISS-20260512T044438272Z-RESOURCE-OWNER-SUMMARY-REPORTS-TESTR-03F3C18F
title: "Resource owner summary reports TestReport print machine result as maybe leak"
area: static-check
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "stdlib/std/test/report.nepl, nepl-core/src/resource, stdlib/tests/btreemap.n.md"
---

# ISS-20260512T044438272Z-RESOURCE-OWNER-SUMMARY-REPORTS-TESTR-03F3C18F: Resource owner summary reports TestReport print machine result as maybe leak

## 概要

On current main, stdlib/tests/btreemap.n.md fails all 5 doctests at compile phase with resource.owner.maybe_leak in checks_print_machine__TestReport__TestReport__imp. The failure reproduces without the BTreeMap API split changes, so it is a ResourceIR/TestReport owner-flow issue rather than a collection regression.

## 対象

- `stdlib/std/test/report.nepl, nepl-core/src/resource, stdlib/tests/btreemap.n.md`

## 根拠

- 未記入

## 問題

On current main, stdlib/tests/btreemap.n.md fails all 5 doctests at compile phase with resource.owner.maybe_leak in checks_print_machine__TestReport__TestReport__imp. The failure reproduces without the BTreeMap API split changes, so it is a ResourceIR/TestReport owner-flow issue rather than a collection regression.

## 影響

Stdlib collection suites that use std/test report printing cannot be used as broad regression checks under the strict owner gate. Weakening owner checks would hide real leaks, so the TestReport return/print owner boundary must be represented accurately.

## 修正方針

Investigate checks_print_machine and ResourceIR owner summaries for TestReport-returning stdout/report helpers. Preserve the single-owner TestReport contract, and fix either the std/test API shape or the checker summary/drop handling so returned report owners are not also reported as leaking temporaries.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/btreemap.n.md --no-tree -o tmp/btreemap-testreport-owner-fixed.json -j 1 --dist web/dist and focused std/test report tests without resource.owner.maybe_leak.
