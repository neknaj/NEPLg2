---
id: ISS-20260505T064000205Z-RESOURCE-IR-SOURCE-FIXTURES-FAIL-ON--EDEA5603
title: "Resource IR source fixtures fail on warning-only shadow diagnostics"
area: core
status: open
resolved: false
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
