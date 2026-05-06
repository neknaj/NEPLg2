---
id: ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744
title: "Resource IR lowering treats callable value references as uninitialized locals"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage_hir_scope.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T104708173Z-RESOURCE-IR-LOWERING-TREATS-CALLABLE-02721744: Resource IR lowering treats callable value references as uninitialized locals

## 概要

Bare callable references and generated lambda function values can reach Resource IR as HirExprKind::Var when no local binding exists. The Resource IR lowerer currently treats every Var as a local read, so function values such as add_op or generated lambda symbols are checked as uninitialized locals.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/tests/resource_ir.rs, tests/compiler/functions.n.md`

## 根拠

GitHub Actions run 25428627798 shows tests/compiler/functions.n.md::doctest#8 and #12 failing with resource.cell.uninit for add_op/sub_op and generated lambda symbols. The Resource IR initialized checker already marks ResourceOp::FunctionValue outputs initialized, but lower.rs emits Read/LocalRead for unresolved-local Var names.

## 問題

Resource IR cell-state authority cannot be correct while callable value references are represented as fallback local places. This creates false resource.cell.uninit diagnostics and can pressure later work to weaken cell checks instead of preserving function-value semantics.

## 影響

First-class functions, bare function references in branch returns, and lambda literals can fail before codegen. Because the old move_check fallback has been removed, this is now a hard compiler boundary regression.

## 修正方針

Teach Resource IR lowering to distinguish local value reads from callable value references: if a Var name has no active local place but is a known callable symbol in the HIR module, lower it as ResourceOp::FunctionValue with the callable effect. Keep local function-typed variables as local reads so alias/cell checks remain precise.

## 対応結果

Resource IR lowering now resolves unresolved-local `HirExprKind::Var` by typed callable identity. A source-level callable name is matched to the unique HIR function with the same `origin_name` and function type, then lowered as `ResourceOp::FunctionValue` with the canonical specialized symbol and effect. Local bindings still win, so function-typed local variables remain ordinary `Read` / `LocalRead` operations and continue to participate in cell / alias checks.

The HIR-to-ResourceIR coverage gate was updated to understand the same function-value rule. It now counts bare callable `Var` expressions as function values only when no local binding shadows the name, so coverage diagnostics remain a guard against missing lowering rather than a source of false mismatches. The callable-name / scope tracking state was split into `coverage_hir_scope.rs` to keep the coverage walker responsibilities explicit.

関連 stage: [静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行)

## 検証

Add a Resource IR regression proving bare function references in branch returns and generated lambda values produce no CellUnavailable diagnostics. Run focused cargo tests, nodesrc issue/source policy checks, trunk build, and the affected doctest subset.

確認済み:

- `cargo fmt --check`
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_initializes_bare_callable_var_references -- --nocapture`
- `cargo test -p nepl-core --test functions function_return -- --nocapture`
- `node nodesrc/test_resource_gate_order.js`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/functions.n.md -o tmp/resource-callable-functions-tests.json --runner wasm --no-tree -j 1 --assert-io`: 23/24 passed. The affected callable false positives `doctest#8` and `doctest#12` passed; the remaining `doctest#23` failure is the existing stdlib raw-memory-backed `effect.pure.calls_impure` issue.
