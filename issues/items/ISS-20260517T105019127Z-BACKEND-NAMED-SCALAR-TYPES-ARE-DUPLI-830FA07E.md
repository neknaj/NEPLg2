---
id: ISS-20260517T105019127Z-BACKEND-NAMED-SCALAR-TYPES-ARE-DUPLI-830FA07E
title: "backend named scalar types are duplicated as strings across typecheck layout and codegen"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/backend_scalar_type.rs, nepl-core/src/types.rs, nepl-core/src/layout.rs, nepl-core/src/wasm_shared.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/codegen_llvm/type_map.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/intrinsic.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T105019127Z-BACKEND-NAMED-SCALAR-TYPES-ARE-DUPLI-830FA07E: backend named scalar types are duplicated as strings across typecheck layout and codegen

## 概要

The compiler treats i64, u64, and f64 as TypeKind::Named string literals in multiple consumers. Storage layout, WASM signatures, LLVM type mapping, and scalar intrinsic type lookup each duplicate the spelling and classification rules.

## 対象

- `nepl-core/src/backend_scalar_type.rs, nepl-core/src/types.rs, nepl-core/src/layout.rs, nepl-core/src/wasm_shared.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/src/codegen_llvm/type_map.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/intrinsic.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 関連計画: [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- `i64` / `u64` / `f64` は TypeCtx では `TypeKind::Named` として流れるが、layout / WASM / LLVM / scalar intrinsic がそれぞれ別の string branch で backend scalar として再分類していた。
- `u32` / `u64` scalar intrinsic は named type id を prefix checker が直接 lookup/register しており、同じ spelling と TypeCtx 登録規則が compiler subsystem ごとに散らばっていた。

## 問題

The compiler treats i64, u64, and f64 as TypeKind::Named string literals in multiple consumers. Storage layout, WASM signatures, LLVM type mapping, and scalar intrinsic type lookup each duplicate the spelling and classification rules.

## 影響

A scalar backend type can be added or changed in one compiler subsystem without the others noticing. That can produce unsound static layout, mismatched backend signatures, or incorrect intrinsic typing without Rust match exhaustiveness catching the drift.

## 修正方針

Introduce a typed BackendScalarType domain that owns supported named scalar spelling and storage semantics, then make layout, wasm signature, llvm type map, and typecheck scalar intrinsic lookup consume that enum.

## 解決内容

- `BackendScalarType::{U32, I64, U64, F64}` を追加し、source spelling、TypeCtx 登録、TypeKind / TypeExpr 分類、storage size / align、WASM/LLVM scalar category を同一 enum domain に集約した。
- `layout.rs`、`wasm_shared.rs`、`codegen_wasm.rs`、`codegen_llvm.rs`、`codegen_llvm/type_map.rs`、`typecheck/prefix_check.rs` を `BackendScalarType` 消費に置き換え、named scalar 文字列の再分類を削除した。
- `types.rs` の named scalar Copy eligibility も同じ enum domain から導出し、`u32` / `u64` が backend scalar として copy-eligible になるようにした。
- `nepl-core/tests/intrinsic.rs` の i64/f64 raw load/store regression を現在の source capability 設計に合わせ、raw 操作を検証する 2 ケースだけ compiler-owned raw boundary helper で実行するようにした。
- source responsibility policy に `BackendScalarType` の存在、consumer 接続、旧 direct string branch の再導入禁止を追加した。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core backend_scalar_type::tests --lib -- --nocapture`
- `cargo test -p nepl-core --test layout -- --nocapture`
- `cargo test -p nepl-core scalar_intrinsic --lib -- --nocapture`
- `cargo test -p nepl-core --test intrinsic -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `trunk build`
