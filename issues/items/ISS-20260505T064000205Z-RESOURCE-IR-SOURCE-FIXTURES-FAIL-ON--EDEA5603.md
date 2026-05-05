---
id: ISS-20260505T064000205Z-RESOURCE-IR-SOURCE-FIXTURES-FAIL-ON--EDEA5603
title: "Resource IR source fixtures fail on warning-only shadow diagnostics"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/tests/resource_ir.rs,nepl-core/src/typecheck/binding_rules.rs"
---

# ISS-20260505T064000205Z-RESOURCE-IR-SOURCE-FIXTURES-FAIL-ON--EDEA5603: Resource IR source fixtures fail on warning-only shadow diagnostics

## 概要

Resource IR integration tests that should exercise resource diagnostics can fail before lowering because typecheck_resource_source asserts diagnostics.is_empty and treats Resolve::ShadowSameSignatureCallable warnings from stdlib imports as hard failures.

## 対象

- `nepl-core/tests/resource_ir.rs,nepl-core/src/typecheck/binding_rules.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell -- --nocapture` が `typecheck_resource_source` の `checked.diagnostics.is_empty()` assertion で失敗した。
- diagnostics は `Severity::Warning` の `Resolve::ShadowSameSignatureCallable` だけで、Resource IR lowering / initialized move check まで到達していない。
- 同じ test file には `typecheck_resource_source_allow_warnings` があるが、source fixture ごとに warning を許容するかどうかの基準が整理されていない。

## 問題

Resource IR integration tests that should exercise resource diagnostics can fail before lowering because typecheck_resource_source asserts diagnostics.is_empty and treats Resolve::ShadowSameSignatureCallable warnings from stdlib imports as hard failures.

## 影響

Focused Resource IR regression verification is blocked by warning-only shadow diagnostics, making memory-safety tests brittle and hiding whether the Resource IR check itself passed or failed.

## 修正方針

Test harness should distinguish warning diagnostics from errors for fixtures that intentionally import broad stdlib modules, or the fixture imports should be narrowed so resource-check tests only fail on actual typecheck errors. The fix must not silence real errors.

## 検証

Run resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell and confirm it reaches Resource IR diagnostics while continuing to reject Severity::Error.

## 対応

- `typecheck_resource_source_with_target` の停止条件を `diagnostics.is_empty()` から `Severity::Error` が存在しないことへ変更した。
- Resource IR tests は typecheck warning の有無ではなく、HIR lowering 後の resource diagnostics を検証するため、warning-only diagnostics は test harness で許容する。
- 既存の `typecheck_resource_source_allow_warnings` は同じ責務になったため削除し、call site を通常 helper に統一した。
- real typecheck error は引き続き assertion で停止するため、warning の許容が型検査失敗の隠蔽にはならない。

## 2026-05-05 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_repeated_str_view_observer_results -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_raw_region_owner_to_str_after_str_leaf_split -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 180 秒で timeout。今回の helper 修正で直接触った旧 warning 許容 call site と再現 test は focused で確認済み。
