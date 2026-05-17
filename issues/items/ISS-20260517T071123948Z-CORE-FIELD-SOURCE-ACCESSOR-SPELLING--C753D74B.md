---
id: ISS-20260517T071123948Z-CORE-FIELD-SOURCE-ACCESSOR-SPELLING--C753D74B
title: "core field source accessor spelling is duplicated outside FieldAccessorKind"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/intrinsic_kinds.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/source_capability/owner_aggregate/field_imports.rs"
---

# ISS-20260517T071123948Z-CORE-FIELD-SOURCE-ACCESSOR-SPELLING--C753D74B: core field source accessor spelling is duplicated outside FieldAccessorKind

## 概要

Resource IR aggregate lowering/coverage still classify source-level field::get/get_ref calls with direct get/get_ref strings, while source capability imports keep a separate CoreFieldAccessorMember enum. The field accessor semantic domain therefore remains split between source member names and intrinsic names.

## 対象

- `nepl-core/src/intrinsic_kinds.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/source_capability/owner_aggregate/field_imports.rs`

## 根拠

- `FieldAccessorKind` は `Get` / `GetRef` / `Put` の semantic domain だが、修正前は intrinsic spelling だけを所有していた。
- `resource/coverage_hir_projection.rs` と `resource/lower_aggregate.rs` は source-level `field::get` / `field::get_ref` を local string branch で再分類していた。
- `source_capability/owner_aggregate/field_imports.rs` は private `CoreFieldAccessorMember` enum を持ち、同じ `get` / `get_ref` / `put` spelling を `FieldAccessorKind` の外で管理していた。

## 問題

Resource IR aggregate lowering/coverage still classify source-level field::get/get_ref calls with direct get/get_ref strings, while source capability imports keep a separate CoreFieldAccessorMember enum. The field accessor semantic domain therefore remains split between source member names and intrinsic names.

## 影響

Field accessor semantics can drift between source import proof, HIR/typecheck, Resource IR coverage, and Resource IR lowering. Adding or changing a field accessor can update intrinsic handling while source-level get/get_ref paths remain stale and outside enum/match exhaustiveness.

## 修正方針

Make FieldAccessorKind own source member spelling as well as intrinsic spelling. Consume FieldAccessorKind::from_core_field_member_name from Resource IR field source consumers and owner aggregate field import proof, and add source policy checks that reject direct get/get_ref classification in those consumers.

## 対応内容

- `FieldAccessorKind::from_core_field_member_name` と `FieldAccessorKind::core_field_member_name` を追加し、`core/field` source member spelling も `FieldAccessorKind` が所有するようにした。
- Resource IR aggregate coverage / lowering は source-level `field::get` / `field::get_ref` の分類に `FieldAccessorKind::from_core_field_member_name` を使う。
- owner aggregate field import proof から private `CoreFieldAccessorMember` を削除し、selective / alias / open import の member 判定を shared `FieldAccessorKind` に接続した。
- source policy に、Resource IR と owner aggregate field import proof が local `get` / `get_ref` / `put` spelling classifier を再導入しない検査を追加した。

## 検証

cargo test -p nepl-core field_accessor --lib -- --nocapture; cargo test -p nepl-core owner_aggregate_boundary_accepts_field_alias_import_call_head --lib -- --nocapture; cargo test -p nepl-core --test resource_ir resource_ir_lowering_treats_compiler_field_load_as_field_read -- --exact --nocapture; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/test_resource_checker_responsibility.js
