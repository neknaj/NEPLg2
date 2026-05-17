---
id: ISS-20260517T050529675Z-FIELD-ACCESSOR-INTRINSIC-DETECTION-D-D864665B
title: "field accessor intrinsic detection duplicates string spelling outside FieldAccessorKind"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/binding_rules.rs
---

# ISS-20260517T050529675Z-FIELD-ACCESSOR-INTRINSIC-DETECTION-D-D864665B: field accessor intrinsic detection duplicates string spelling outside FieldAccessorKind

## 概要

typecheck/binding_rules.rs detects core field accessor wrappers by matching get_field/get_field_ref/set_field strings directly, while the rest of typecheck carries FieldAccessorKind as the typed domain. The spelling-to-kind mapping therefore lives outside the enum that should own it.

## 対象

- `nepl-core/src/typecheck/binding_rules.rs`

## 根拠

- `nepl-core/src/typecheck/model.rs` には `FieldAccessorKind::{Get, GetRef, Put}` があり、field accessor の typed domain は既に存在していた。
- `nepl-core/src/typecheck/binding_rules.rs` の `detect_field_accessor_fn` はこの enum を使わず、`intrin.name == "get_field"` / `"get_field_ref"` / `"set_field"` を直接 match していた。
- そのため accessor kind と intrinsic spelling の対応が enum の外へ漏れ、追加・変更時に wrapper 検出だけ更新漏れする構造だった。

## 問題

typecheck/binding_rules.rs detects core field accessor wrappers by matching get_field/get_field_ref/set_field strings directly, while the rest of typecheck carries FieldAccessorKind as the typed domain. The spelling-to-kind mapping therefore lives outside the enum that should own it.

## 影響

Adding or changing a field accessor intrinsic can update FieldAccessorKind users without updating wrapper detection, leaving owner aggregate field access and typecheck behavior dependent on scattered string matches instead of enum/match exhaustiveness.

## 修正方針

Move field accessor intrinsic spelling classification onto FieldAccessorKind and make detect_field_accessor_fn consume that typed classifier. Add source policy coverage so binding_rules.rs cannot reintroduce direct get_field/get_field_ref/set_field match arms.

## 検証

cargo check -p nepl-core; cargo test -p nepl-core field_accessor --lib -- --nocapture; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

## 対応内容

- `FieldAccessorKind::from_intrinsic_name` と `FieldAccessorKind::intrinsic_name` を追加し、spelling と kind の対応を enum 側に集約した。
- `detect_field_accessor_fn` は `FieldAccessorKind::from_intrinsic_name` を呼ぶだけにし、binding rules から直接 string match を削除した。
- `field_apply.rs` の `get_field` / `get_field_ref` 生成も `FieldAccessorKind::intrinsic_name` を使うようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に、binding rules へ直接 spelling match を戻さない policy を追加した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core field_accessor_intrinsic_names_round_trip_through_kind --lib -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
