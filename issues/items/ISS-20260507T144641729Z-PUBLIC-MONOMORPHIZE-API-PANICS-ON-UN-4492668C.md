---
id: ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C
title: "Public monomorphize API panics on unresolved trait calls"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/monomorphize.rs, nepl-core/src/compiler.rs"
---

# ISS-20260507T144641729Z-PUBLIC-MONOMORPHIZE-API-PANICS-ON-UN-4492668C: Public monomorphize API panics on unresolved trait calls

## 概要

The compile pipeline now calls monomorphize_with_unresolved_trait_calls and converts unresolved trait calls into diagnostics. However, nepl-core still publicly exports monomorphize(ctx, module), which calls monomorphize_internal(..., true). If any unresolved trait call remains, assert_no_trait_calls panics with an internal compiler error instead of returning diagnostics. The API is currently unused inside the repository, but it remains public through nepl-core/src/lib.rs.

## 対象

- `nepl-core/src/monomorphize.rs, nepl-core/src/compiler.rs`

## 根拠

- `nepl-core/src/lib.rs` は `pub mod monomorphize;` により monomorphize module を公開している。
- `nepl-core/src/monomorphize.rs` の `pub fn monomorphize(ctx, module)` は `monomorphize_internal(ctx, module, true).0` を呼ぶ。
- `monomorphize_internal(..., true)` は unresolved trait call が残る場合に `assert_no_trait_calls` を呼び、`panic!("internal compiler error: unresolved trait call remained after monomorphize: ...")` する。
- `nepl-core/src/compiler.rs` の compile preparation は `monomorphize_with_unresolved_trait_calls` を使い、unresolved trait call を diagnostic へ変換している。
- repository 内の直接利用は現時点で compile pipeline 側の diagnostic-returning API に寄っており、panic API は未使用だが public API として残っている。

## 問題

The compile pipeline now calls monomorphize_with_unresolved_trait_calls and converts unresolved trait calls into diagnostics. However, nepl-core still publicly exports monomorphize(ctx, module), which calls monomorphize_internal(..., true). If any unresolved trait call remains, assert_no_trait_calls panics with an internal compiler error instead of returning diagnostics. The API is currently unused inside the repository, but it remains public through nepl-core/src/lib.rs.

## 影響

A public compiler API can still crash instead of reporting a typed diagnostic. This conflicts with the diagnostic redesign and the policy that compiler failures should be checkable rather than panic-based. Future selfhost or tool integrations may accidentally call the shorter API and reintroduce unresolved-trait panics outside the guarded compile pipeline.

## 修正方針

Remove the panic-based public API or change it to return Result / unresolved diagnostics. Keep a single monomorphize entry point that exposes unresolved trait calls structurally, and make compiler.rs map them to BackendDiagnosticCode::TraitCallUnresolved. Add a source-policy regression rejecting panic-based unresolved-trait handling in monomorphize public APIs.

## 検証

Add a focused regression that constructs or compiles a module with an unresolved trait call through the public monomorphize boundary and confirms diagnostics/structured errors are returned rather than panic. Keep existing compile pipeline unresolved-trait diagnostics green in GitHub Actions.
