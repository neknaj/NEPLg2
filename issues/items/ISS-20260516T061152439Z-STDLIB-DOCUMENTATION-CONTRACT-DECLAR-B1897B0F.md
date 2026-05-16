---
id: ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F
title: "Stdlib documentation contract declaration doctest baseline regressed to 1039 on main"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-05-16
updated: 2026-05-16
target: "stdlib/alloc/collections/**/*.nepl, stdlib/core/mem/types.nepl"
---

# ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F: Stdlib documentation contract declaration doctest baseline regressed to 1039 on main

## 概要

origin/main currently reports declarationNoDoctest=1039 while nodesrc/test_stdlib_documentation_contract.js freezes the baseline at 1032. This is not caused by the BTreeMap report fixture branch; current, HEAD, and origin/main have the same count. The new gaps are concentrated in owner-bearing collection/memory APIs such as BTreeMapInsertError, BTreeSetInsertError, StackPop accessors, VecPartition helpers, VecStorage/OwnedBuffer, VecPop accessors, and region_token_raw_ref.

## 対象

- `stdlib/alloc/collections/**/*.nepl, stdlib/core/mem/types.nepl`

## 根拠

- 未記入

## 問題

origin/main currently reports declarationNoDoctest=1039 while nodesrc/test_stdlib_documentation_contract.js freezes the baseline at 1032. This is not caused by the BTreeMap report fixture branch; current, HEAD, and origin/main have the same count. The new gaps are concentrated in owner-bearing collection/memory APIs such as BTreeMapInsertError, BTreeSetInsertError, StackPop accessors, VecPartition helpers, VecStorage/OwnedBuffer, VecPop accessors, and region_token_raw_ref.

## 影響

The global source-policy runner fails before unrelated static-check work can be reported cleanly, and owner-bearing stdlib APIs lack executable documentation examples even though docs/doctests are part of the API contract.

## 修正方針

Add meaningful n.md-style declaration doctests for the listed public APIs instead of raising the baseline. Keep the policy baseline at 1032 or lower, and use TestReport stdout fixtures where runtime behavior is expected.

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, focused doctests for the updated stdlib files, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check --dir issues, and git diff --check.
