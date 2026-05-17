---
id: ISS-20260517T061045507Z-RESOURCE-FIELD-ACCESSOR-INTRINSIC-PR-9951F2BE
title: "resource field accessor intrinsic proof duplicates typecheck spelling"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/resource/lower_aggregate.rs
---

# ISS-20260517T061045507Z-RESOURCE-FIELD-ACCESSOR-INTRINSIC-PR-9951F2BE: resource field accessor intrinsic proof duplicates typecheck spelling

## 概要

Field accessor intrinsic spelling is represented by FieldAccessorKind in typecheck, but Resource IR coverage and lowering still check helper_base_name(name) against get_field and get_field_ref directly. The Resource IR consumer therefore duplicates intrinsic spelling instead of consuming a shared typed intrinsic domain.

## 対象

- `nepl-core/src/resource/lower_aggregate.rs`

## 根拠

- `typecheck/model.rs` の `FieldAccessorKind` は `get_field` / `get_field_ref` / `set_field` の spelling と arity を所有していた。
- しかし `resource/coverage_hir_projection.rs` と `resource/lower_aggregate.rs` は `helper_base_name(name) != "get_field"` / `"get_field_ref"` で同じ intrinsic spelling を再分類していた。
- Resource IR の aggregate projection proof は typechecked HIR の field accessor intrinsic と同じ contract を消費するべきであり、consumer 側で spelling を重複管理すると静的検査の proof が drift する。

## 問題

Field accessor intrinsic spelling is represented by FieldAccessorKind in typecheck, but Resource IR coverage and lowering still check helper_base_name(name) against get_field and get_field_ref directly. The Resource IR consumer therefore duplicates intrinsic spelling instead of consuming a shared typed intrinsic domain.

## 影響

A field accessor intrinsic spelling change can update typecheck without updating Resource IR aggregate projection proof. This weakens static-check correctness because typed HIR construction and Resource IR proof can drift.

## 修正方針

Move FieldAccessorKind to a shared crate-level intrinsic kind module, update typecheck and Resource IR consumers to use it, and add source policy coverage that rejects direct get_field/get_field_ref checks in Resource IR aggregate projection consumers.

## 対応内容

- `FieldAccessorKind` を `typecheck/model.rs` から crate-level `intrinsic_kinds.rs` へ移した。
- typecheck 側の binding / prefix checker は共有 enum を従来どおり消費する。
- Resource IR の `coverage_hir_projection.rs` / `lower_aggregate.rs` は `FieldAccessorKind::from_intrinsic_name(helper_base_name(name))` を使い、direct `"get_field"` / `"get_field_ref"` 比較を削除した。
- source policy に、Resource IR aggregate projection consumer が shared enum を使い、direct spelling check を再導入しないことを追加した。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core field_accessor_intrinsic --lib -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test neplg2 field_accessor_intrinsic_arg_arity_has_type_code -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
