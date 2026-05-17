---
id: ISS-20260517T060320513Z-PREFIX-CORE-INTRINSIC-RESULT-TYPING--0B7AACB4
title: "prefix core intrinsic result typing still uses string branches"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/prefix_check.rs
---

# ISS-20260517T060320513Z-PREFIX-CORE-INTRINSIC-RESULT-TYPING--0B7AACB4: prefix core intrinsic result typing still uses string branches

## 概要

typecheck/prefix_check.rs still computes result types for core intrinsics such as size_of, align_of, load, store, callsite_span, and unreachable by direct intrin.name string branches. Scalar and field accessor intrinsics have typed domains, but these core intrinsic contracts remain outside enum/match exhaustiveness.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs`

## 根拠

- `typecheck/prefix_check.rs` は `size_of` / `align_of` / `load` / `store` / `callsite_span` / `unreachable` の result type を `intrin.name` への direct string branch で決めていた。
- scalar intrinsic と field accessor intrinsic は既に `ScalarIntrinsicKind` / `FieldAccessorKind` に移っているため、core intrinsic だけが spelling と result kind を enum domain の外に残していた。
- `callsite_span` の type arg arity diagnostic も branch 内に閉じており、core intrinsic contract を網羅的に監査できない構造だった。

## 問題

typecheck/prefix_check.rs still computes result types for core intrinsics such as size_of, align_of, load, store, callsite_span, and unreachable by direct intrin.name string branches. Scalar and field accessor intrinsics have typed domains, but these core intrinsic contracts remain outside enum/match exhaustiveness.

## 影響

Adding or changing a core intrinsic can update effect classification or lowering without updating type result rules. The static checker itself remains harder to audit because intrinsic spelling, type-argument contract, and result kind are not held in a typed domain.

## 修正方針

Introduce a CoreIntrinsicKind enum in typecheck/model.rs that owns core intrinsic spelling and result kind. Make prefix_check.rs classify once through CoreIntrinsicKind and derive result types from exhaustive matches. Add model tests and source policy coverage that rejects direct intrin.name branches for these core intrinsics.

## 対応内容

- `CoreIntrinsicKind` と `CoreIntrinsicResultKind` を `typecheck/model.rs` に追加した。
- `CoreIntrinsicKind::from_intrinsic_name` / `intrinsic_name` / `result_kind` で core intrinsic spelling と result contract を所有するようにした。
- `prefix_check.rs` は `CoreIntrinsicKind::from_intrinsic_name` で分類し、`core_intrinsic_type_id` の exhaustive match から result type を導出する。
- `nodesrc/test_static_check_boundary_responsibility.js` に source policy を追加し、core intrinsic の `intrin.name == ...` branch 再導入を拒否する。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core core_intrinsic --lib -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test neplg2 callsite_span_type_arg_arity_has_type_code -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test intrinsic intrinsic_size_and_align_direct -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
