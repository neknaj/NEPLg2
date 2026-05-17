---
id: ISS-20260517T070337891Z-OWNER-AGGREGATE-INTRINSIC-EVIDENCE-D-4A69BCFB
title: "owner aggregate intrinsic evidence duplicates field accessor spelling"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/owner_aggregate/evidence.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T070337891Z-OWNER-AGGREGATE-INTRINSIC-EVIDENCE-D-4A69BCFB: owner aggregate intrinsic evidence duplicates field accessor spelling

## 概要

owner_aggregate_intrinsic_evidence still matches get_field/get_field_ref directly even after FieldAccessorKind became the shared typed intrinsic domain. Source capability proof can drift from typecheck and Resource IR field accessor contracts.

## 対象

- `nepl-core/src/source_capability/owner_aggregate/evidence.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `FieldAccessorKind` は `nepl-core/src/intrinsic_kinds.rs` に移動済みで、typecheck と Resource IR は field accessor intrinsic spelling / arity をこの enum domain から消費している。
- しかし `nepl-core/src/source_capability/owner_aggregate/evidence.rs` の `owner_aggregate_intrinsic_evidence` は `helper_base_name(symbol)` を `get_field` / `get_field_ref` の direct string branch で分類しており、source capability proof だけが shared intrinsic kind の外に残っていた。

## 問題

owner_aggregate_intrinsic_evidence still matches get_field/get_field_ref directly even after FieldAccessorKind became the shared typed intrinsic domain. Source capability proof can drift from typecheck and Resource IR field accessor contracts.

## 影響

Changing field accessor intrinsic spelling or adding variants can leave owner aggregate source evidence stale, weakening static-check maintainability and allowing duplicated string policy outside enum/exhaustive-match control.

## 修正方針

Classify intrinsic evidence through FieldAccessorKind::from_intrinsic_name and accept only the read/reference variants via exhaustive match. Add source policy checks that prevent direct get_field/get_field_ref spelling from returning to owner aggregate evidence.

## 対応内容

- `owner_aggregate_intrinsic_evidence` を `FieldAccessorKind::from_intrinsic_name(helper_base_name(symbol))` に接続した。
- `FieldAccessorKind::{Get, GetRef, Put}` を `match` で網羅し、owner aggregate field boundary evidence は既存仕様どおり `Get` / `GetRef` のみに限定した。
- `loader.rs` に `set_field` intrinsic が owner aggregate field boundary evidence にならない regression を追加した。
- `nodesrc/test_static_check_boundary_responsibility.js` に source policy を追加し、owner aggregate evidence で direct `get_field` / `get_field_ref` spelling が再導入されることを拒否する。

## 検証

cargo test -p nepl-core owner_aggregate_boundary_accepts_intrinsic_field_evidence --lib -- --nocapture
cargo test -p nepl-core owner_aggregate_boundary_rejects_set_field_intrinsic_evidence --lib -- --nocapture
node nodesrc/test_static_check_boundary_responsibility.js
