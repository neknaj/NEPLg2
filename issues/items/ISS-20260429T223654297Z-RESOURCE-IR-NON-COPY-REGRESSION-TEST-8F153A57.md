---
id: ISS-20260429T223654297Z-RESOURCE-IR-NON-COPY-REGRESSION-TEST-8F153A57
title: "Resource IR non-Copy regression tests use skeleton lowering with mismatched TypeCtx"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/tests/resource_ir.rs, nodesrc/test_resource_ir_test_harness_policy.js"
---

# ISS-20260429T223654297Z-RESOURCE-IR-NON-COPY-REGRESSION-TEST-8F153A57: Resource IR non-Copy regression tests use skeleton lowering with mismatched TypeCtx

## 概要

Resource IR initialized/moved regression tests construct a custom non-Copy type in one TypeCtx but lower the HIR through lower_hir_module_skeleton, which creates a fresh TypeCtx. The HIR still contains TypeId values from the custom context, so lowering aggregate construction can index outside the fresh arena and panic before the static checker reports CellUnavailable.

## 対象

- `nepl-core/tests/resource_ir.rs, nepl-core/src/resource/lower.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir -- --nocapture` で `resource_ir_check_reports_non_copy_use_after_move` と `resource_ir_check_reports_read_after_drop` が `types.rs:475` の `TypeCtx::resolve_id` 範囲外 access で panic していた。
- backtrace では `lower_hir_module_skeleton` が fresh `TypeCtx::new()` を使い、custom non-Copy `Owned` の `TypeId(8)` を含む HIR を lowering して `aggregate_construct_field_offsets` に到達していた。
- そのため Resource IR の `CellUnavailable` 診断を確認する前に test harness が落ち、静的検証の回帰監視になっていなかった。

## 問題

Resource IR initialized/moved regression tests construct a custom non-Copy type in one TypeCtx but lower the HIR through lower_hir_module_skeleton, which creates a fresh TypeCtx. The HIR still contains TypeId values from the custom context, so lowering aggregate construction can index outside the fresh arena and panic before the static checker reports CellUnavailable.

## 影響

Static verification regressions can fail by panic instead of checking Resource IR diagnostics. This hides whether the initialized/moved checker is actually enforcing non-Copy use-after-move and read-after-drop, weakening regression coverage for memory safety.

## 修正方針

Use the same TypeCtx for Resource IR lowering whenever HIR contains non-builtin TypeId values, and add a source-level regression policy so non-Copy Resource IR tests cannot regress to lower_hir_module_skeleton.

## 対応

- custom non-Copy `Owned` 型を使う 2 つの Resource IR initialized/moved regression は、`lower_hir_module_skeleton` ではなく `lower_hir_module(&module, &types)` で lowering するようにした。
- `nodesrc/test_resource_ir_test_harness_policy.js` を追加し、対象 2 test が custom `TypeCtx` で lowering すること、fresh `TypeCtx` を作る skeleton lowering に戻らないことを固定した。

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_check_reports_non_copy_use_after_move resource_ir_check_reports_read_after_drop -- --nocapture; cargo test -p nepl-core --test resource_ir -- --nocapture

- `cargo test -p nepl-core --test resource_ir resource_ir_check_reports -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: `124 passed`
- `cargo test -p nepl-core --test effects -- --nocapture`: `21 passed`
- `node nodesrc/test_resource_ir_test_harness_policy.js`: passed
- `rustfmt --check nepl-core/tests/resource_ir.rs`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
