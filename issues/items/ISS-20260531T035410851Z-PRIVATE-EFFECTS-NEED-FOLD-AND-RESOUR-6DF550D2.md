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

## 2026-06-01 region provenance checkpoint

`PrivateEffectRegion::UnsealedIntrinsic` を追加し、`InternalEffect::{PrivateState, PrivateCache}` と Resource IR `EffectOp::{PrivateState, PrivateCache}` に region provenance を保持するようにした。

この region は mask 済み region ではなく、trusted intrinsic 由来だが fresh/non-escape proof がまだない private effect を表す。`internal_effect_surface_fold` は従来どおり `Impure` に倒し、pure function 内の `PrivateCache` / `PrivateState` は dedicated diagnostic で fail closed に拒否する。

Resource summary body hash は private effect operation に加えて region provenance も hash する。あわせて private effect policy hash を `neplg2-private-effect-policy-v2` に上げ、古い `.neplproof` / `.neplmeta` artifact が region なしの private effect policy として再利用されないようにした。

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo test -p nepl-core private_cache --lib -- --nocapture`
- `cargo test -p nepl-core private_effect --lib -- --nocapture`
- `cargo test -p nepl-core resource_effect_gate --lib -- --nocapture`
- `cargo test -p nepl-core resource_function_body_hash --lib -- --nocapture`

残件:

- `UnsealedIntrinsic` ではなく fresh private region id を発行する backend/cache representation。
- region が public type、return value、global/public field、raw pointer、stats/clear/ref API へ escape しないことの Resource IR proof。
- proof 済み region だけを Pure へ mask する accepted regression。
