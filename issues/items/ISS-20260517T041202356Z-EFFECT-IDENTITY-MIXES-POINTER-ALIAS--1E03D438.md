---
id: ISS-20260517T041202356Z-EFFECT-IDENTITY-MIXES-POINTER-ALIAS--1E03D438
title: "effect_identity mixes pointer alias table with raw identity table"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/effect_identity.rs, nepl-core/src/resource/effect_pointer_alias.rs, nepl-core/src/resource/effect_place_prefix.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T041202356Z-EFFECT-IDENTITY-MIXES-POINTER-ALIAS--1E03D438: effect_identity mixes pointer alias table with raw identity table

## 概要

Resource checker responsibility policy fails because effect_identity.rs has grown past the split limit. The file owns both raw identity groups and pointer alias table mechanics, making the static-check implementation harder to audit.

## 対象

- `nepl-core/src/resource/effect_identity.rs, nepl-core/src/resource/effect_pointer_alias.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `effect_identity.rs has 463 lines; responsibility split limit is 420` で失敗した。
- `effect_identity.rs` が raw identity group と pointer alias table の両方を保持しており、Resource IR effect proof の監査単位が混在していた。

## 問題

Resource checker responsibility policy fails because effect_identity.rs has grown past the split limit. The file owns both raw identity groups and pointer alias table mechanics, making the static-check implementation harder to audit.

## 影響

Large mixed responsibility in Resource IR identity tracking increases the chance of static-check bugs and makes raw identity vs non-owning pointer alias behavior harder to review independently.

## 修正方針

- `RawPointerAliasTable` を `effect_pointer_alias.rs` に分離し、利用側は `effect_identity` 経由ではなく `effect_pointer_alias` から明示的に import する。
- identity / pointer alias の双方が使う prefix replacement / suffix extraction / unique place append は `effect_place_prefix.rs` に分離し、pointer alias が identity 実装へ循環依存しない形にする。
- `nodesrc/test_resource_checker_responsibility.js` に新しい責務単位と行数上限を追加し、再統合や肥大化を検出できるようにする。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
