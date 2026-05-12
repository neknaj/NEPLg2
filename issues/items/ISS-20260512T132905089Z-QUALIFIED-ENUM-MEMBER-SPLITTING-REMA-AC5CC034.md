---
id: ISS-20260512T132905089Z-QUALIFIED-ENUM-MEMBER-SPLITTING-REMA-AC5CC034
title: "Qualified enum member splitting remains duplicated across compiler stages"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/qualified_name.rs; nepl-core/src/typecheck/syntax_helpers.rs; nepl-core/src/resource/variant_name.rs; nepl-core/src/codegen_wasm.rs; nepl-core/src/codegen_llvm.rs; nepl-core/src/runtime_helpers.rs; nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260512T132905089Z-QUALIFIED-ENUM-MEMBER-SPLITTING-REMA-AC5CC034: Qualified enum member splitting remains duplicated across compiler stages

## 概要

Qualified enum member tail and leading qualifier parsing were implemented separately in typecheck, Resource IR, wasm codegen, LLVM codegen, and runtime helper lookup. This kept first-vs-last separator rules distributed across compiler stages.

## 対象

- `nepl-core/src/qualified_name.rs`
- `nepl-core/src/typecheck/syntax_helpers.rs`
- `nepl-core/src/typecheck/call_reduction.rs`
- `nepl-core/src/resource/variant_name.rs`
- `nepl-core/src/codegen_wasm.rs`
- `nepl-core/src/codegen_llvm.rs`
- `nepl-core/src/runtime_helpers.rs`
- `nodesrc/test_static_check_boundary_responsibility.js`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `typecheck/syntax_helpers.rs` had local leading qualifier / variant tail helpers.
- `resource/variant_name.rs` had a separate tail helper for Resource IR enum payload place keys.
- `codegen_wasm.rs` and `codegen_llvm.rs` had local `variant.rfind("::")` logic in both enum tag and payload lookup.
- `runtime_helpers.rs` had local `name.rfind("::")` logic for helper base name stripping.
- `typecheck/call_reduction.rs` had local `rfind("::")` logic for expected enum type inference from match arms.

## 問題

Qualified-name parsing was not owned by one module. Even though each local copy used a reasonable rule in isolation, the compiler had no single authority for the distinction between leading qualifier split and member-tail split.

## 影響

A future change can update typecheck or Resource IR variant normalization without updating codegen or runtime helper lookup, causing typed static-check facts and generated backend tags/payloads to disagree for qualified enum variants or helper names.

## 修正方針

Introduce a crate-level qualified_name helper module for leading qualifier split, member tail, and prefix/tail split; migrate typecheck, Resource IR, wasm/LLVM codegen, runtime helper lookup, and source policy to use it.

## 対応記録

- `nepl-core/src/qualified_name.rs` を追加し、`split_leading_qualifier`、`member_tail`、`split_member_tail` を定義した。
- typecheck の `split_qualified_name` / `variant_member_tail` は crate 共通 helper の wrapper にした。
- Resource IR の `variant_name_tail` は crate 共通 `member_tail` を使うようにした。
- wasm / LLVM backend の enum tag / payload lookup は local `rfind("::")` をやめ、crate 共通 `member_tail` を使うようにした。
- runtime helper base name stripping と match arm expected type inference も crate 共通 helper へ移行した。
- static-check boundary policy で、`qualified_name.rs` 以外の Rust source に `rfind("::")` / `rsplit("::")` / `splitn(2, "::")` を再導入しないことを検査するようにした。

## 検証

- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core qualified_name::tests -- --nocapture`: passed
- `cargo test -p nepl-core typecheck::syntax_helpers::tests -- --nocapture`: passed
- `cargo test -p nepl-core --test import_clause alias_qualified_enum_match_arm_uses_variant_member_tail -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
