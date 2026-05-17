---
id: ISS-20260517T064500300Z-RESOURCE-LAYOUT-INTRINSIC-PROOFS-DUP-81984703
title: "Resource layout intrinsic proofs duplicate size_of spelling outside shared intrinsic kind"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/intrinsic_kinds.rs, nepl-core/src/typecheck/model.rs, nepl-core/src/resource/lower_layout_intrinsic.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower_raw_address_return_util.rs, nodesrc/test_static_check_boundary_responsibility.js, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T064500300Z-RESOURCE-LAYOUT-INTRINSIC-PROOFS-DUP-81984703: Resource layout intrinsic proofs duplicate size_of spelling outside shared intrinsic kind

## 概要

Resource IR evaluates size_of/align_of constants by matching helper_base_name strings locally, while CoreIntrinsicKind is currently typecheck-local. The proof side and result-typing side can drift because layout intrinsic spelling and value semantics are not owned by one typed enum domain.

## 対象

- `nepl-core/src/intrinsic_kinds.rs, nepl-core/src/typecheck/model.rs, nepl-core/src/resource/lower_layout_intrinsic.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower_raw_address_return_util.rs, nodesrc/test_static_check_boundary_responsibility.js, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `nepl-core/src/typecheck/model.rs` には `CoreIntrinsicKind` があり、`size_of` / `align_of` の result type は typed enum domain から導出されていた。
- 一方で `nepl-core/src/resource/lower_layout_intrinsic.rs` は `helper_base_name(name)` を直接 `"size_of"` / `"align_of"` に match し、同じ spelling を Resource IR 側で再分類していた。
- `nepl-core/src/resource/lower_raw_address.rs` と `nepl-core/src/resource/lower_raw_address_return_util.rs` は raw address offset constant 評価で `helper_base_name(name) == "size_of"` を直接判定し、layout constant proof が `CoreIntrinsicKind` から切り離されていた。

## 問題

Resource IR evaluates size_of/align_of constants by matching helper_base_name strings locally, while CoreIntrinsicKind is currently typecheck-local. The proof side and result-typing side can drift because layout intrinsic spelling and value semantics are not owned by one typed enum domain.

## 影響

Static checking can become unsound or incomplete if a layout intrinsic spelling/result contract is changed in typecheck but Resource IR continues using stale local string checks. This violates the current policy that compiler proofs should be source/IR-derived and enum-exhaustive rather than module-specific allowlists.

## 修正方針

Move core intrinsic classification that is shared across passes into crate-level intrinsic_kinds.rs, expose a layout intrinsic subset with exhaustive value semantics, and make typecheck and Resource IR consume the same typed classifier. Add source policy coverage to prevent direct size_of/align_of string branches from returning to Resource IR.

## 検証

Run cargo fmt/check, targeted Resource IR layout/raw-address regressions, and nodesrc responsibility policy checks.

## 対応結果

- `CoreIntrinsicKind` / `CoreIntrinsicResultKind` を `typecheck/model.rs` から crate-level `intrinsic_kinds.rs` へ移し、typecheck と Resource IR の両方が同じ typed enum domain を消費するようにした。
- `CoreIntrinsicKind::layout_i32_value` を追加し、`SizeOf` / `AlignOf` の compile-time layout value を exhaustive match から導出するようにした。`Load` / `Store` / `CallsiteSpan` / `Unreachable` は layout value を持たない branch として明示的に扱う。
- `resource/lower_layout_intrinsic.rs` は `"size_of"` / `"align_of"` の直接 match をやめ、`CoreIntrinsicKind::from_intrinsic_name(...).layout_i32_value(...)` に統一した。
- raw address offset constant 評価は `i32_const_from_size_of_call` を削除し、call / intrinsic の両方を `layout_intrinsic_i64_value(_from_callee)` 経由にした。これにより `size_of` だけの個別 proof ではなく、layout intrinsic の typed proof を共有する。
- source policy に、Resource IR 側で `size_of` / `align_of` の direct string branch を再導入しない検査を追加した。

## 実施した検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core core_intrinsic --lib -- --nocapture`: pass
- `cargo test -p nepl-core core_layout_intrinsic_value_is_kind_owned --lib -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_layout_intrinsics_use_shared_core_intrinsic_kind -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_symbolic_mem_ptr_add_offset -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_literal_arithmetic_helper_zero_offset -- --exact --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_untracked_literal_helper_zero_offset_for_first_store -- --exact --nocapture`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: CRLF warnings only
