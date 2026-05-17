---
id: ISS-20260517T003825703Z-RESOURCE-PLACE-SKELETON-STILL-CLASSI-5ED30216
title: "Resource place skeleton still classifies address add by local string match"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/address_projection.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage_hir_place.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T003825703Z-RESOURCE-PLACE-SKELETON-STILL-CLASSI-5ED30216: Resource place skeleton still classifies address add by local string match

## 概要

After field-load address projection was centralized, lower.rs and coverage_hir_place.rs still recognize address add projections through local
ame == \

## 対象

- `nepl-core/src/resource/address_projection.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage_hir_place.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `lower.rs` の `place_from_expr_skeleton` は storage-offset place を作るときに `name == "add"` を直接見ていた。
- `coverage_hir_place.rs` の place coverage も同じく `name == "add"` を直接見ており、直前に導入した `address_projection.rs` の共有 classifier の外に残っていた。
- place skeleton は borrow/deref/storage-offset の根拠になるため、field-load lowering だけを共有 classifier にしても、place lowering / coverage 側に文字列分類が残ると検査プログラム自体の一貫性が崩れる。

## 問題

After field-load address projection was centralized, lower.rs and coverage_hir_place.rs still recognize address add projections through local
ame == \

## 影響

The Resource IR place skeleton can drift from HIR coverage and field-load lowering if address projection syntax changes. Because these places feed borrow/deref/storage-offset coverage, memory-safety diagnostics can become dependent on duplicated string matching rather than the shared static-check classifier.

## 修正方針

Extend resource/address_projection.rs with address-projection predicates for intrinsic place skeletons and storage-offset projection extraction. Use those helpers from lower.rs and coverage_hir_place.rs, and add source policy guards against reintroducing local
ame == \

## 検証

Run focused Resource IR coverage regression, cargo check/fmt, resource responsibility policy, static-check policy, issues check, and diff check.

## 対応内容

- `address_projection.rs` に `intrinsic_is_address_projection` と `storage_offset_base_and_offset` を追加した。
- `lower.rs` の `place_from_expr_skeleton` は shared classifier から `ResourceOffset::{Known, Unknown}` を受け取り、local `name == "add"` / literal offset parsing を持たない形にした。
- `coverage_hir_place.rs` は `intrinsic_is_address_projection` を使い、place coverage の address-add 判定を shared classifier へ接続した。
- `nodesrc/test_resource_checker_responsibility.js` に lower / coverage_hir_place の local `name == "add"` 再導入禁止を追加した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_borrow_and_deref_places -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_treats_compiler_field_load_as_field_read -- --exact --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt -p nepl-core --check`
- `node nodesrc/test_resource_checker_responsibility.js`
