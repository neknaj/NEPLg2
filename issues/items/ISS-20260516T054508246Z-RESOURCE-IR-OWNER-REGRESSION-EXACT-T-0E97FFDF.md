---
id: ISS-20260516T054508246Z-RESOURCE-IR-OWNER-REGRESSION-EXACT-T-0E97FFDF
title: "Resource IR owner regression exact tests fail on current main after Stage 6 owner model changes"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/tests/resource_ir.rs, nepl-core/src/resource/**, stdlib/core/mem/**, stdlib/alloc/collections/vec/**"
---

# ISS-20260516T054508246Z-RESOURCE-IR-OWNER-REGRESSION-EXACT-T-0E97FFDF: Resource IR owner regression exact tests fail on current main after Stage 6 owner model changes

## 概要

Several exact resource_ir owner regression tests fail on current origin/main. Known identity callbacks are blocked by owner aggregate constructor restrictions on origin/main and progress to raw-memory provenance diagnostics on the BTreeMap proof branch; double-dealloc diagnostics report Moved/OwnerUnavailable rather than the intended Freed classification; several stale stdlib fixtures still reference removed or restricted Stage 6 owner surfaces.

## 対象

- `nepl-core/tests/resource_ir.rs, nepl-core/src/resource/**, stdlib/core/mem/**, stdlib/alloc/collections/vec/**`

## 根拠

- `cargo test -p nepl-core resource_ir_owner -- --nocapture` on the BTreeMap proof branch ran 102 `resource_ir` owner tests and failed 8. The BTreeMap-focused regression added in `ISS-20260516T042051521Z-BTREEMAP-FOCUSED-DOCTESTS-STILL-HIDE-87F9DD7B` passed, so the failure is broader than the current doctest migration.
- A detached worktree at `origin/main` (`98965f1d`) reproduces representative failures:
  - `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_region_ptr_through_known_identity_callback -- --exact --nocapture`
  - `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_double_dealloc -- --exact --nocapture`
  - `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view -- --exact --nocapture`
- On `origin/main`, known identity / ByteBuf cases stop at `OwnerAggregateConstructorRestricted` and related type errors from Stage 6 owner aggregate restrictions. On the BTreeMap proof branch, explicit constructor evidence moves some of those paths past typecheck and exposes raw-memory provenance diagnostics instead.
- `resource_ir_owner_check_reports_double_dealloc` fails on both `origin/main` and the BTreeMap proof branch because the second dealloc path reports `OwnerUnavailable { state: Moved }` rather than the intended freed-owner classification. That makes the test expectation stale or the diagnostic classification incomplete.
- [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) requires Resource IR owner tests to remain authoritative while Stage 6 moves `MemPtr` to non-owning pointer and `RegionToken` / storage to free-obligation owners.

## 問題

Several exact resource_ir owner regression tests fail on current origin/main. Known identity callbacks are blocked by owner aggregate constructor restrictions on origin/main and progress to raw-memory provenance diagnostics on the BTreeMap proof branch; double-dealloc diagnostics report Moved/OwnerUnavailable rather than the intended Freed classification; several stale stdlib fixtures still reference removed or restricted Stage 6 owner surfaces.

## 影響

Resource IR owner regression coverage is not fully authoritative while these exact tests fail. Static-check safety work can miss whether a failure is a real compiler regression, a stale fixture, or an incomplete Stage 6 proof.

## 修正方針

Audit the failing exact tests one by one against the current Stage 6 model. For each case, either update stale fixtures to current RegionToken/OwnedBuffer/non-owning MemPtr APIs or fix the compiler proof so source/type/IR evidence proves the intended safety property without stdlib/module allowlists.

## 検証

Run each failing exact resource_ir owner regression individually on current main, then run the focused Resource IR owner suite after the fixes.
