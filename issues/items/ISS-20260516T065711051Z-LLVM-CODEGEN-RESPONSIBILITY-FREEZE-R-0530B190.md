---
id: ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190
title: "LLVM codegen responsibility freeze regressed on main"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/codegen_llvm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md"
---

# ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190: LLVM codegen responsibility freeze regressed on main

## 概要

After syncing remote main during ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59, node nodesrc/test_parser_backend_responsibility_policy.js reports nepl-core/src/codegen_llvm.rs at 4217 lines while the responsibility freeze limit is 4189. This is independent from the WASM string-data/aggregate split.

## 対象

- `nepl-core/src/codegen_llvm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- 未記入

## 問題

After syncing remote main during ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59, node nodesrc/test_parser_backend_responsibility_policy.js reports nepl-core/src/codegen_llvm.rs at 4217 lines while the responsibility freeze limit is 4189. This is independent from the WASM string-data/aggregate split.

## 影響

The parser/backend source-policy runner remains red after the WASM backend split, so unrelated compiler work can keep carrying an architecture warning and LLVM backend responsibilities can continue accumulating in the root file.

## 修正方針

Follow doc/neplg2/parser_backend_responsibility_split_plan.md B3. Move a coherent LLVM backend responsibility such as raw body handling, aggregate lowering, or value/local mapping into a dedicated module, then lower or keep the root line limit instead of raising it.

## 検証

Run node nodesrc/test_parser_backend_responsibility_policy.js, focused LLVM codegen tests, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check --dir issues, and git diff --check.
