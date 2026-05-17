---
id: ISS-20260517T111300558Z-SCALAR-INTRINSIC-BACKEND-LOWERING-DU-DEB6C1B2
title: "Scalar intrinsic backend lowering duplicates typecheck-only signature domain"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/intrinsic_kinds.rs, nepl-core/src/typecheck/model.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/passes/codegen_precheck.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T111300558Z-SCALAR-INTRINSIC-BACKEND-LOWERING-DU-DEB6C1B2: Scalar intrinsic backend lowering duplicates typecheck-only signature domain

## 概要

ScalarIntrinsicKind currently lives inside typecheck/model.rs, so backend lowering and codegen precheck cannot consume the typed signature domain and instead duplicate scalar intrinsic names such as i64_to_u64/u64_to_i64 with direct string branches.

## 対象

- `nepl-core/src/intrinsic_kinds.rs, nepl-core/src/typecheck/model.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/passes/codegen_precheck.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `ScalarIntrinsicKind` / `ScalarIntrinsicType` は `typecheck/model.rs` の `pub(super)` item だったため、typecheck module の外では利用できなかった。
- `wasm_shared.rs`、`passes/codegen_precheck.rs`、`codegen_wasm.rs`、`codegen_llvm.rs` が scalar intrinsic spelling をそれぞれ持ち、backend が typecheck 済み signature を文字列で再分類していた。
- LLVM precheck は `i32_to_f32` / reinterpret 系 scalar intrinsic を typecheck では認めるのに support list では拒否する状態で、typecheck と backend support の drift が実際に発生していた。

## 問題

ScalarIntrinsicKind currently lives inside typecheck/model.rs, so backend lowering and codegen precheck cannot consume the typed signature domain and instead duplicate scalar intrinsic names such as i64_to_u64/u64_to_i64 with direct string branches.

## 影響

Scalar intrinsic spelling, arity, representation, and backend support can drift between typecheck, wasm lowering, llvm lowering, and precheck. That weakens the static-check migration because the compiler proves a scalar intrinsic signature in one subsystem but reinterprets it by string in another.

## 修正方針

Move ScalarIntrinsicKind and ScalarIntrinsicType to the shared intrinsic_kinds module, add a typed backend lowering operation owned by the scalar intrinsic enum, make typecheck/prefix_check, wasm_shared, codegen_precheck, codegen_wasm, and codegen_llvm consume that enum, and add source-policy regressions that reject scalar intrinsic string duplication outside the shared enum domain.

## 対応内容

- `ScalarIntrinsicKind` / `ScalarIntrinsicType` を `typecheck/model.rs` から `intrinsic_kinds.rs` へ移し、typecheck と backend が同じ enum domain を消費する構造にした。
- `ScalarIntrinsicBackendOp` を追加し、scalar intrinsic の backend lowering semantics を spelling ではなく enum の `backend_op()` に集約した。
- WASM support precheck、LLVM support precheck、WASM lowering、LLVM lowering を `ScalarIntrinsicKind::from_intrinsic_name` と `backend_op()` に接続し、scalar intrinsic 名の direct list / branch を backend consumer から削除した。
- LLVM lowering に `i32_to_f32` / `reinterpret_i32_f32` / `reinterpret_f32_i32` の実装を追加し、typecheck が認める scalar intrinsic と LLVM backend support の drift を解消した。
- source policy に、typecheck/model への再ローカライズ禁止、backend consumer の scalar spelling 重複禁止、WASM/LLVM の typed scalar intrinsic lowering 接続確認を追加した。

## 検証

cargo fmt -p nepl-core --check; cargo check -p nepl-core; focused nepl-core intrinsic/codegen tests; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues; trunk build

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core intrinsic_kinds::tests --lib -- --nocapture`: 9 passed
- `cargo test -p nepl-core --test neplg2 llvm_scalar_intrinsics_use_shared_backend_lowering -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test codegen_diagnostics llvm_precheck_reports_unknown_intrinsic_type_code -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test intrinsic -- --nocapture`: 4 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
