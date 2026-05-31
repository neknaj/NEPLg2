---
id: ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2
title: "Private effects need fold and Resource summary hash integration"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
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

## 2026-06-01 checkpoint

`PrivateState` / `PrivateCache` を `InternalEffect` と Resource IR `EffectOp` に追加し、mask boundary がない pure function では dedicated diagnostic で拒否するようにした。

Resource summary body hash は `PrivateState` / `PrivateCache` operation を hash する。さらに `ResourceOp::FunctionValue` に `ResourceFunctionValueKind::{Plain, Memoized}` を追加し、memoized function value が plain function value と同じ body hash へ落ちないようにした。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo test -p nepl-core private_cache_effect --lib -- --nocapture`
- `cargo test -p nepl-core private_state_effect --lib -- --nocapture`
- `cargo test -p nepl-core private_effect --lib -- --nocapture`
- `cargo test -p nepl-core resource_function_body_hash_tracks_memoized_function_value_kind --lib -- --nocapture`

残件:

- `PrivateCache rho` / `PrivateState rho` の fresh region と non-escape proof。
- proven mask boundary の accepted regression。
- private region id を持つ backend/cache operation への一般化。
