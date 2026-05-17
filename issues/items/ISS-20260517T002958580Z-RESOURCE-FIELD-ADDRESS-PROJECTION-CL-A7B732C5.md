---
id: ISS-20260517T002958580Z-RESOURCE-FIELD-ADDRESS-PROJECTION-CL-A7B732C5
title: "Resource field address projection classifier is duplicated between coverage and lowering"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/coverage_hir_projection_aggregate.rs, nepl-core/src/resource/lower_aggregate_selector.rs, nepl-core/src/resource/address_projection.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T002958580Z-RESOURCE-FIELD-ADDRESS-PROJECTION-CL-A7B732C5: Resource field address projection classifier is duplicated between coverage and lowering

## 概要

Resource IR lowering and HIR coverage each parse compiler field address expressions by matching add(base, literal_offset) locally. The duplicated helper-name and literal-offset logic makes the coverage proof and the lowering proof independent programs instead of one shared classifier.

## 対象

- `nepl-core/src/resource/coverage_hir_projection_aggregate.rs, nepl-core/src/resource/lower_aggregate_selector.rs, nepl-core/src/resource/address_projection.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `coverage_hir_projection_aggregate.rs` と `lower_aggregate_selector.rs` が、それぞれ `add(base, literal_offset)` と direct call の helper 名分類を持っていた。
- Resource lowering completeness gate は HIR coverage と Resource IR lowering の一致を証明するための検査であり、同じ address projection rule を別々に実装すると gate 自体の信頼性が下がる。
- 静的検査大規模修正の方針では、検査プログラム自体も enum / match と shared classifier に寄せ、文字列分岐の重複で証明が形骸化しないようにする必要がある。

## 問題

Resource IR lowering and HIR coverage each parse compiler field address expressions by matching add(base, literal_offset) locally. The duplicated helper-name and literal-offset logic makes the coverage proof and the lowering proof independent programs instead of one shared classifier.

## 影響

A later change can update lowering without updating coverage, or vice versa, causing Resource IR lowering completeness diagnostics to drift from the actual memory-safety proof. This leaves static-check correctness dependent on duplicated string and literal matching rather than a shared enum/match boundary.

## 修正方針

Introduce a small shared Resource IR address projection classifier for add(base, non-negative literal offset), use it from both lower_aggregate_selector and coverage_hir_projection_aggregate, and add source policy guards that forbid reintroducing local callee_base_name/add parsing in those modules.

## 検証

Run focused Resource IR field-load coverage test, cargo check, resource responsibility policy, issues check, and diff check.

## 対応内容

- `resource/address_projection.rs` を追加し、`AddressProjectionPrimitive::Add`、`non_negative_i32_literal`、`compiler_field_address_base_and_offset` を共有 classifier として定義した。
- `lower_aggregate.rs` と `coverage_hir_projection.rs` は共有 classifier を使う形にし、coverage/lowering が別々の field-address parser を持たないようにした。
- `lower_aggregate_selector.rs` は aggregate selector の担当に戻し、literal selector 判定だけを共有 `non_negative_i32_literal` へ接続した。
- `coverage_hir_projection_aggregate.rs` から local `add` / `callee_base_name` classifier を削除した。
- `nodesrc/test_resource_checker_responsibility.js` に `address_projection.rs` の存在、line limit、共有 classifier 使用、local `add` classifier 再導入禁止を追加した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_treats_compiler_field_load_as_field_read -- --exact --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt -p nepl-core --check`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
