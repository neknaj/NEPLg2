---
id: ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2
title: "Private effects need fold and Resource summary hash integration"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/effects.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/resource_summary_value_cache/body_hash.rs"
---

# ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2: Private effects need fold and Resource summary hash integration

## 概要

Adding PrivateAlloc, PrivateState, PrivateCache, and PrivateRegionId changes internal effect semantics and Resource IR bodies, so surface folding, effect diagnostics, and Resource summary value cache invalidation must be updated together.

## 対象

- `nepl-core/src/effects.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/resource_summary_value_cache/body_hash.rs`

## 根拠

- 未記入

## 問題

Adding PrivateAlloc, PrivateState, PrivateCache, and PrivateRegionId changes internal effect semantics and Resource IR bodies, so surface folding, effect diagnostics, and Resource summary value cache invalidation must be updated together.

## 影響

If Private effects are folded directly to Pure or omitted from body/source-capability hashes, memo_call can either bypass escape proof or reuse stale Resource summary values after private effect boundaries change.

## 修正方針

Define private effect row variants, keep them unmasked until a Resource IR boundary proves fresh non-escape, add dedicated diagnostics for unmasked private effects and private state observation, and include Private effect operations/region boundaries in stable body hash and capability policy hash inputs.

## 検証

Tests should reject unmasked PrivateCache in pure functions, accept it only behind a proven mask boundary, report dedicated private-state diagnostics, and invalidate Resource summary cache keys when private effect operations or capability use-sites change.
