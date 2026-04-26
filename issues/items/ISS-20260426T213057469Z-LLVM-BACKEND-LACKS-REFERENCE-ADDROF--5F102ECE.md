---
id: ISS-20260426T213057469Z-LLVM-BACKEND-LACKS-REFERENCE-ADDROF--5F102ECE
title: "LLVM backend lacks reference AddrOf and Deref lowering"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/codegen_llvm.rs, nepl-core/tests/neplg2.rs, tests/compiler/reference_codegen.n.md"
---

# ISS-20260426T213057469Z-LLVM-BACKEND-LACKS-REFERENCE-ADDROF--5F102ECE: LLVM backend lacks reference AddrOf and Deref lowering

## 概要

GitHub Actions run 24967172989 llvm-dual-tests reaches D4106 unsupported expression kind for AddrOf and Deref in drop, drop_overwrite, Vec get_ref, and selfhost CLI args paths.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/passes/codegen_precheck.rs, tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, stdlib/alloc/collections/vec.nepl`

## 根拠

- `nepl-core/src/passes/codegen_precheck.rs` は `HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner)` を子式の検査だけで通している。
- `nepl-core/src/codegen_llvm.rs` は `TypeKind::Reference` を `LlTy::I32` として扱い、`get_field_ref` / `load` / `store` では linear memory pointer を扱える。
- 一方で `lower_hir_expr` には `HirExprKind::AddrOf` / `HirExprKind::Deref` の arm がなく、precheck 後に `CodegenLlvmUnsupportedHir` へ落ちる構造になっていた。
- WASM backend は scalar `AddrOf` を linear memory に一時保存して pointer を返し、aggregate `AddrOf` は既存 handle を返す。`Deref` は scalar load、aggregate byte copy として実装済み。

## 問題

GitHub Actions run 24967172989 llvm-dual-tests reaches D4106 unsupported expression kind for AddrOf and Deref in drop, drop_overwrite, Vec get_ref, and selfhost CLI args paths.

## 影響

Borrow/reference safety features pass the WASM route but cannot be checked through LLVM dual verification, and new borrow/lifetime work can regress LLVM silently until codegen.

## 修正方針

Define LLVM lowering for AddrOf/Deref/reference projection with the same safety preconditions as WASM, or reject unsupported reference cases in codegen_precheck before LLVM lowering with a stable diagnostic.

## 検証

Run focused LLVM compile cases for tests/compiler/drop.n.md, tests/compiler/drop_overwrite.n.md, and Vec get_ref doctests.

## 解決

- LLVM lowering に `HirExprKind::AddrOf` を追加し、aggregate は既存 `i32` handle、scalar は `alloc` で確保した linear memory slot へ store した pointer を返すようにした。
- LLVM lowering に `HirExprKind::Deref` を追加し、scalar は pointer から typed load、`u8` は zero-extend load、aggregate は linear memory byte copy で値 object を返すようにした。
- 全 `HirExprKind` が明示的に lower されるようになったため、到達不能になった catch-all の unsupported arm は削除した。
- LLVM IR emit の Rust 回帰テストで、scalar `&i32` / `*x` と aggregate `&Pair` が D4106 へ落ちず LLVM IR に下りることを固定した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test neplg2 llvm_reference -- --nocapture`: 2/2 passed
- `cargo test -p nepl-core --test neplg2 llvm -- --nocapture`: 4/4 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --no-tree -o tmp/reference-codegen-after-llvm-ref.json -j 1`: 2/2 passed
- `node nodesrc/tests.js -i tests/compiler/reference_codegen.n.md --runner llvm --llvm-all --llvm-compile-only -o tmp/llvm-reference-codegen-before.json -j 1`: local clang 未設定のため `failed to execute clang --version`
