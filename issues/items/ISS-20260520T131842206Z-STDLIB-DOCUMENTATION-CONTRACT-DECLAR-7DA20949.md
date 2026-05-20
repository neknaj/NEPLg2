---
id: ISS-20260520T131842206Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-7DA20949
title: "Stdlib documentation contract declaration doctest baseline regressed to 1036"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/core/**, stdlib/alloc/**, stdlib/std/**, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260520T131842206Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-7DA20949: Stdlib documentation contract declaration doctest baseline regressed to 1036

## 概要

The global stdlib documentation contract currently reports declarationNoDoctest=1036 while the frozen baseline is 1032. This failure is outside the selfhost checker files changed for ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D, but it blocks the aggregate source-policy runner from reporting cleanly.

## 対象

- `stdlib/core/**, stdlib/alloc/**, stdlib/std/**, nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` fails with `stdlib declaration doctest gaps increased: 1036 > 1032`.
- `node nodesrc/run_source_policy_regressions.js` reaches `nodesrc/test_stdlib_documentation_contract.js` and stops on the same failure.
- The selfhost checker timeout fix touches `stdlib/neplg2/core/check/**`, while the documentation contract scans only `stdlib/core`, `stdlib/alloc`, and `stdlib/std`; this regression is therefore separated from `ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D`.

## 問題

The global stdlib documentation contract currently reports declarationNoDoctest=1036 while the frozen baseline is 1032. This failure is outside the selfhost checker files changed for ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D, but it blocks the aggregate source-policy runner from reporting cleanly.

## 影響

Global source-policy verification remains noisy and executable documentation coverage for public stdlib declarations has regressed below the enforced baseline.

## 修正方針

Audit the four new declaration doctest gaps, add meaningful n.md-style doctests instead of relaxing the baseline, and keep the documentation contract baseline at 1032 or lower.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js and node nodesrc/run_source_policy_regressions.js.
