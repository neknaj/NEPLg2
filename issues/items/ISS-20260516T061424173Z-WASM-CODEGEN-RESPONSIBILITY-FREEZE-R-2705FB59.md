---
id: ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59
title: "WASM codegen responsibility freeze regressed on main"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md"
---

# ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59: WASM codegen responsibility freeze regressed on main

## 概要

origin/main currently has nepl-core/src/codegen_wasm.rs at 2582 lines while nodesrc/test_parser_backend_responsibility_policy.js freezes the limit at 2574. This is not caused by the BTreeMap ResourceIR branch; current, HEAD, and origin/main all report the same 2582-line count.

## 対象

- `nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- 未記入

## 問題

origin/main currently has nepl-core/src/codegen_wasm.rs at 2582 lines while nodesrc/test_parser_backend_responsibility_policy.js freezes the limit at 2574. This is not caused by the BTreeMap ResourceIR branch; current, HEAD, and origin/main all report the same 2582-line count.

## 影響

The parser/backend source-policy runner warns even in unrelated compiler work, and the WASM backend continues to accumulate responsibilities in a large root file instead of moving instruction emission, aggregate lowering, match lowering, raw body handling, or helper lowering into planned submodules.

## 修正方針

Follow doc/neplg2/parser_backend_responsibility_split_plan.md B2 rather than raising the line limit. Move a coherent WASM backend responsibility into a dedicated module and lower or keep the root limit.

## 検証

Run node nodesrc/test_parser_backend_responsibility_policy.js, relevant WASM codegen tests, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check --dir issues, and git diff --check.
