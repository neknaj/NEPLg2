---
id: ISS-20260430T070449791Z-FALLIBLE-OWNER-EFFECTS-DO-NOT-RESERV-32CC9198
title: "Fallible owner effects do not reserve owners before Result refinement"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260430T070449791Z-FALLIBLE-OWNER-EFFECTS-DO-NOT-RESERV-32CC9198: Fallible owner effects do not reserve owners before Result refinement

## 概要

PendingVariantOwnerEffects delays owner consumption/return until a Result match arm is selected, but the source owner remains usable before that refinement. A caller can ignore or delay matching a fallible owner effect and may reuse an owner that is consumed on the success variant.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

PendingVariantOwnerEffects delays owner consumption/return until a Result match arm is selected, but the source owner remains usable before that refinement. A caller can ignore or delay matching a fallible owner effect and may reuse an owner that is consumed on the success variant.

## 影響

This can become a false negative for memory safety: a fallible dealloc/realloc wrapper may consume an owner at runtime, while the Resource IR checker still permits using the original owner until the Result is matched.

## 修正方針

Represent fallible owner effects as a reserved/path-dependent owner state until the result is refined. Before a matching Result arm or equivalent refinement, direct use/dealloc/return of the reserved source must be rejected or require explicit handling of all variants.

## 検証

Add Resource IR regressions where dealloc_ptr/realloc_ptr result is ignored or matched after reusing the original owner, and assert resource.owner diagnostics are emitted.
