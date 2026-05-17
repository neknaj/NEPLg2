---
id: ISS-20260517T185446545Z-SYNTHETIC-HIR-RAW-AND-ADDRESS-INTRIN-0E618808
title: "synthetic HIR raw and address intrinsics bypass typed intrinsic domains"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-18
target: "nepl-core/src/typecheck/call_reduction.rs; nepl-core/src/typecheck/field_apply.rs; nepl-core/src/typecheck/prefix_check.rs; nepl-core/src/passes/drop_insertion.rs; nepl-core/src/intrinsic_kinds.rs; nepl-core/src/scalar_primitives.rs; nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T185446545Z-SYNTHETIC-HIR-RAW-AND-ADDRESS-INTRIN-0E618808: synthetic HIR raw and address intrinsics bypass typed intrinsic domains

## 概要

typecheck and drop insertion synthesize HIR Intrinsic nodes with direct add/load/store strings, bypassing CoreIntrinsicKind and I32ArithmeticPrimitive even though static-check proof consumers now rely on typed enum domains.

## 対象

- `nepl-core/src/typecheck/call_reduction.rs; nepl-core/src/typecheck/field_apply.rs; nepl-core/src/typecheck/prefix_check.rs; nepl-core/src/passes/drop_insertion.rs; nepl-core/src/intrinsic_kinds.rs; nepl-core/src/scalar_primitives.rs; nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/typecheck/call_reduction.rs` が dotted field read の合成HIRで `add` と `load` を直接文字列生成していた。
- `nepl-core/src/typecheck/field_apply.rs` と `nepl-core/src/typecheck/prefix_check.rs` が field put / prefix assignment の合成HIRで `add` と `store` を直接文字列生成していた。
- `nepl-core/src/passes/drop_insertion.rs` が enum payload drop elaboration の合成HIRで `load` と `add` を直接文字列生成していた。
- これらのproducerは Resource IR / effect / typecheck consumer が使う `CoreIntrinsicKind` / `I32ArithmeticPrimitive` のtyped domainを迂回しており、checker自体の分岐driftをRustの`match`網羅性で検出しにくかった。

## 問題

typecheck and drop insertion synthesize HIR Intrinsic nodes with direct add/load/store strings, bypassing CoreIntrinsicKind and I32ArithmeticPrimitive even though static-check proof consumers now rely on typed enum domains.

## 影響

Field access, nested aggregate load, assignment lowering, and drop insertion can drift from the compiler-wide intrinsic/effect/resource proof domains. This weakens match exhaustiveness and makes static-check code itself easier to regress.

## 修正方針

Add enum-owned spelling accessors for the remaining synthetic intrinsic constructors and replace direct string construction. Add source policy coverage that rejects reintroducing direct add/load/store intrinsic string construction outside the typed domains.

## 対応内容

- `I32ArithmeticPrimitive::base_name` を追加し、source-level arithmetic spellingを`I32ArithmeticPrimitive` enum側へ集約した。
- typecheckのdotted field read / field put / prefix assignmentで合成する`add` / `load` / `store` intrinsicを、`I32ArithmeticPrimitive` / `CoreIntrinsicKind` から生成するようにした。
- drop insertionのenum payload drop elaborationで合成する`load` / `add` intrinsicも同じtyped domainから生成するようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に、合成HIR producerへ `name: "add/load/store".to_string()` が戻ることを拒否するsource policyを追加した。
- `doc/neplg2/static_check_complexity_reduction_plan.md` にStage 6 compiler-core進捗として反映した。

## 検証

- `rg -n 'name: "add"\.to_string\(\)|name: "load"\.to_string\(\)|name: "store"\.to_string\(\)' nepl-core\src -g '*.rs'`: no matches
- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core i32_arithmetic_source_spellings_round_trip -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core field_accessor_intrinsic -- --nocapture`: passed
- `cargo test -p nepl-core drop --test drop -- --nocapture`: 18 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
