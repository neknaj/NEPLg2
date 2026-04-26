---
id: ISS-20260426T213057469Z-LLVM-BACKEND-LACKS-REFERENCE-ADDROF--5F102ECE
title: "LLVM backend lacks reference AddrOf and Deref lowering"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/codegen_llvm.rs, nepl-core/src/passes/codegen_precheck.rs, tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, stdlib/alloc/collections/vec.nepl"
---

# ISS-20260426T213057469Z-LLVM-BACKEND-LACKS-REFERENCE-ADDROF--5F102ECE: LLVM backend lacks reference AddrOf and Deref lowering

## 概要

GitHub Actions run 24967172989 llvm-dual-tests reaches D4106 unsupported expression kind for AddrOf and Deref in drop, drop_overwrite, Vec get_ref, and selfhost CLI args paths.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/passes/codegen_precheck.rs, tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, stdlib/alloc/collections/vec.nepl`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-tests reaches D4106 unsupported expression kind for AddrOf and Deref in drop, drop_overwrite, Vec get_ref, and selfhost CLI args paths.

## 影響

Borrow/reference safety features pass the WASM route but cannot be checked through LLVM dual verification, and new borrow/lifetime work can regress LLVM silently until codegen.

## 修正方針

Define LLVM lowering for AddrOf/Deref/reference projection with the same safety preconditions as WASM, or reject unsupported reference cases in codegen_precheck before LLVM lowering with a stable diagnostic.

## 検証

Run focused LLVM compile cases for tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, and Vec get_ref doctests.
