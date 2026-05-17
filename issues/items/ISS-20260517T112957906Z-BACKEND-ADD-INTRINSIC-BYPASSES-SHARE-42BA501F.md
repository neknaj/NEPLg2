---
id: ISS-20260517T112957906Z-BACKEND-ADD-INTRINSIC-BYPASSES-SHARE-42BA501F
title: "Backend add intrinsic bypasses shared arithmetic primitive domain"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/scalar_primitives.rs, nepl-core/src/resource/scalar_primitive.rs, nepl-core/src/wasm_shared.rs, nepl-core/src/passes/codegen_precheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs"
---

# ISS-20260517T112957906Z-BACKEND-ADD-INTRINSIC-BYPASSES-SHARE-42BA501F: Backend add intrinsic bypasses shared arithmetic primitive domain

## 概要

WASM/LLVM backend support and lowering still classify the add intrinsic by direct string checks after scalar intrinsic domains were centralized. Resource IR has a typed I32ArithmeticPrimitive domain for add/sub/mul, but backend code keeps a separate add-only spelling table.

## 対象

- `nepl-core/src/scalar_primitives.rs, nepl-core/src/resource/scalar_primitive.rs, nepl-core/src/wasm_shared.rs, nepl-core/src/passes/codegen_precheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs`

## 根拠

- `wasm_shared.rs` は backend intrinsic support を `ScalarIntrinsicKind` 等の enum domain と `name == "add"` の direct string guard の混在で判定していた。
- `passes/codegen_precheck.rs` も LLVM backend intrinsic support で同じ `name == "add"` を持っていた。
- `codegen_wasm.rs` / `codegen_llvm.rs` は `#intrinsic "add"` の lowering をそれぞれ別の string branch と arity/type check で実装していた。
- Resource IR 側には `I32ArithmeticPrimitive` が存在したが、backend 側から参照できない resource-private module に閉じていたため、同じ primitive spelling が compiler 内で分岐していた。

## 問題

WASM/LLVM backend support and lowering still classify the add intrinsic by direct string checks after scalar intrinsic domains were centralized. Resource IR has a typed I32ArithmeticPrimitive domain for add/sub/mul, but backend code keeps a separate add-only spelling table.

## 影響

Static-check and backend acceptance can drift: source/type/resource proof may classify arithmetic through one enum while codegen precheck or lowering accepts or rejects a different spelling. This violates the enum-first exhaustive-check policy for compiler correctness.

## 修正方針

Move the scalar arithmetic primitive domain to a crate-level shared module or otherwise expose it through a typed compiler domain. Make Resource IR, wasm support, LLVM support, and both backend lowerers consume that enum through exhaustive matches; add policy coverage that forbids direct add string guards in backend consumers.

## 対応内容

- `nepl-core/src/scalar_primitives.rs` を追加し、`I32ArithmeticPrimitive` / `I32ComparisonPrimitive` / `BooleanPrimitive` を crate-level の typed primitive domain とした。
- Resource IR の `resource/scalar_primitive.rs` は enum 本体を持たず、`ResourceCallTarget` / `ResourceI32RelationOp` に接続する adapter だけに縮小した。
- WASM/LLVM の intrinsic support 判定は `I32ArithmeticPrimitive::from_codegen_intrinsic_name` を消費するようにした。
- WASM/LLVM lowering は `I32ArithmeticPrimitive` を受け取り、`match` で `Add` のみを backend codegen intrinsic subset として lower するようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` / `nodesrc/test_resource_checker_responsibility.js` に shared primitive domain、backend consumer、resource adapter の責務境界を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core scalar_primitives --lib -- --nocapture`: 4/4 passed
- `cargo test -p nepl-core wasm_intrinsic_support_uses_i32_arithmetic_codegen_subset --lib -- --nocapture`: 1/1 passed
- `cargo test -p nepl-core llvm_intrinsic_support_uses_i32_arithmetic_codegen_subset --lib -- --nocapture`: 1/1 passed
- `cargo test -p nepl-core --test neplg2 llvm_reference_aggregate_addr_of_lowers -- --exact --nocapture`: 1/1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
