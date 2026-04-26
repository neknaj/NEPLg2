---
id: ISS-20260426T213057295Z-LLVM-DUAL-BACKEND-MISSES-ALLOCATOR-R-EF505BD2
title: "LLVM dual backend misses allocator runtime symbols"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/codegen_llvm.rs, nepl-core/src/runtime_helpers.rs, nodesrc/tests.js"
---

# ISS-20260426T213057295Z-LLVM-DUAL-BACKEND-MISSES-ALLOCATOR-R-EF505BD2: LLVM dual backend misses allocator runtime symbols

## 概要

GitHub Actions run 24967172989 llvm-dual-tests has link_llvm_cli failures such as use of undefined value @alloc_raw__i32__i32__pure across block/generic/string/span doctests.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/runtime_helpers.rs, nodesrc/tests.js`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-tests has link_llvm_cli failures such as use of undefined value @alloc_raw__i32__i32__pure across block/generic/string/span doctests.

## 影響

Programs that allocate aggregates can typecheck and lower to LLVM IR but fail at link time, so LLVM backend parity is not actionable for self-host workloads.

## 修正方針

Ensure LLVM codegen emits or links the allocator/runtime helper symbols required by lowered stdlib/core intrinsics, and add a focused LLVM compile/link test for alloc_raw-backed aggregate construction.

## 検証

node nodesrc/tests.js -i tests/compiler/block_single_line.n.md --runner llvm --llvm-all --no-tree -o tmp/llvm-alloc-runtime.json -j 1 passes the alloc_raw link cases.
