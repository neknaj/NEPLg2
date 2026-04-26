---
id: ISS-20260426T213058045Z-LLVM-DUAL-RUNNER-RETURNS-NULL-OR-CRA-BC38ABEB
title: "LLVM dual runner returns null or crashes for broad run cases"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/codegen_llvm.rs, nepl-cli/src/codegen_llvm.rs, nodesrc/tests.js"
---

# ISS-20260426T213058045Z-LLVM-DUAL-RUNNER-RETURNS-NULL-OR-CRA-BC38ABEB: LLVM dual runner returns null or crashes for broad run cases

## 概要

GitHub Actions run 24967172989 tests-dual-tests has 310 run_llvm_cli return value mismatches with actual null, and tests-dual-stdlib has SIGSEGV or null returns for Option/Result/string/stdio doctests.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-cli/src/codegen_llvm.rs, nodesrc/tests.js`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 tests-dual-tests has 310 run_llvm_cli return value mismatches with actual null, and tests-dual-stdlib has SIGSEGV or null returns for Option/Result/string/stdio doctests.

## 影響

LLVM dual verification currently reports broad runtime parity failure, so successful LLVM compilation is not enough to validate generated programs.

## 修正方針

Audit LLVM entry/return ABI, runtime memory initialization, process exit mapping, and runner return-value extraction; split true runtime crashes from runner capture failures.

## 検証

Run a minimal LLVM return-value smoke suite and representative Option/Result/string doctests with actual numeric returns instead of null or SIGSEGV.
