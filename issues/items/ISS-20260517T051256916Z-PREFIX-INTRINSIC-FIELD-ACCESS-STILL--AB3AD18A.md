---
id: ISS-20260517T051256916Z-PREFIX-INTRINSIC-FIELD-ACCESS-STILL--AB3AD18A
title: "prefix intrinsic field access still branches on field accessor strings"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/prefix_check.rs
---

# ISS-20260517T051256916Z-PREFIX-INTRINSIC-FIELD-ACCESS-STILL--AB3AD18A: prefix intrinsic field access still branches on field accessor strings

## 概要

FieldAccessorKind now owns get_field/get_field_ref/set_field spelling, but typecheck/prefix_check.rs still branches directly on intrin.name == get_field/get_field_ref/set_field for intrinsic result typing and field access handling. This leaves the main intrinsic checker outside the typed field accessor domain.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs`

## 根拠

- `FieldAccessorKind` は `get_field` / `get_field_ref` / `set_field` の intrinsic spelling を所有する enum domain になっていた。
- しかし `nepl-core/src/typecheck/prefix_check.rs` は intrinsic result type の決定と field accessor lowering で `intrin.name == "get_field"` / `"get_field_ref"` / `"set_field"` を直接分岐していた。
- そのため wrapper 検出と prefix intrinsic check の間で spelling drift が起きても、Rust の `match` 網羅性で検出できない状態だった。

## 問題

FieldAccessorKind now owns get_field/get_field_ref/set_field spelling, but typecheck/prefix_check.rs still branches directly on intrin.name == get_field/get_field_ref/set_field for intrinsic result typing and field access handling. This leaves the main intrinsic checker outside the typed field accessor domain.

## 影響

A field accessor intrinsic change can update FieldAccessorKind and wrapper detection while prefix_check keeps stale direct string branches. Static-check behavior then depends on scattered spelling matches instead of enum/match exhaustiveness.

## 修正方針

- `prefix_check.rs` で `FieldAccessorKind::from_intrinsic_name` を 1 回だけ計算し、`Get` / `GetRef` / `Put` の result type と lowering を `match` で分岐する。
- HIR に残す intrinsic name も `FieldAccessorKind::{Get,GetRef}.intrinsic_name()` から生成し、field accessor spelling の所有者を `FieldAccessorKind` に固定する。
- `nodesrc/test_static_check_boundary_responsibility.js` に、`prefix_check.rs` が direct field accessor string branch を再導入しない source policy を追加する。

## 検証

- `cargo fmt -p nepl-core --check`: initial failed by formatting only
- `cargo fmt -p nepl-core`: passed
- `cargo fmt -p nepl-core --check`: passed after formatting
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core field_accessor_intrinsic_names_round_trip_through_kind --lib -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
