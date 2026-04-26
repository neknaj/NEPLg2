---
id: ISS-20260426T213058233Z-LLVM-COMPILE-FAIL-DIAGNOSTICS-LOSE-E-E7DD80E7
title: "LLVM compile_fail diagnostics lose expected ids and spans"
area: core
status: open
resolved: false
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-26
target: "nodesrc/tests.js, nepl-core/src/compiler.rs, nepl-core/src/codegen_llvm.rs, tests/compiler/compile_fail_diag_location.n.md"
---

# ISS-20260426T213058233Z-LLVM-COMPILE-FAIL-DIAGNOSTICS-LOSE-E-E7DD80E7: LLVM compile_fail diagnostics lose expected ids and spans

## 概要

GitHub Actions run 24967172989 llvm-dual-tests reports compile_fail diagnostic id/span mismatches for codegen_diagnostics and compile_fail_diag_location under compile_llvm_cli.

## 対象

- `nodesrc/tests.js, nepl-core/src/compiler.rs, nepl-core/src/codegen_llvm.rs, tests/compiler/compile_fail_diag_location.n.md`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-tests reports compile_fail diagnostic id/span mismatches for codegen_diagnostics and compile_fail_diag_location under compile_llvm_cli.

## 影響

Strict dual testing cannot distinguish real diagnostic regressions from LLVM target diagnostic rewriting, and compile_fail coverage becomes noisy.

## 修正方針

Preserve diagnostic IDs/spans through LLVM compile-fail mode or make the dual runner compare the original typecheck diagnostics before LLVM-specific wrapping.

## 検証

Run tests/compiler/codegen_diagnostics.n.md and tests/compiler/compile_fail_diag_location.n.md with --runner llvm --llvm-all and confirm expected ids/spans match.
