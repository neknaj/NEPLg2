---
id: ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF
title: "SourceCapability needs private cache boundary use-sites"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/source_map.rs; nepl-core/src/source_capability/proof_builder.rs; nepl-core/src/source_capability/private_cache.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/model.rs"
---

# ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF: SourceCapability needs private cache boundary use-sites

## 概要

memo_call must not be accepted through a stdlib name allowlist or raw intrinsic shortcut; trusted private cache operations need exact source proof, region provenance, and policy-hash invalidation.

## 対象

- `nepl-core/src/source_map.rs; nepl-core/src/source_capability/proof_builder.rs; nepl-core/src/source_capability/private_cache.rs; nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/model.rs`

## 根拠

- 未記入

## 問題

memo_call must not be accepted through a stdlib name allowlist or raw intrinsic shortcut; trusted private cache operations need exact source proof, region provenance, and policy-hash invalidation.

## 影響

Without a SourceCapability boundary for private cache use-sites, memo_call can become a special-case bypass that is hard to audit and can stale-hit Resource summary caches after capability or source changes.

## 修正方針

Add SourceCapability use-sites for private cache/private state boundaries, include them in source capability policy hashes, and require Resource IR private cache operations to originate from those trusted use-sites.

## 検証

Tests should show that trusted stdlib memo use-sites are accepted, copied or shifted source spans are rejected, local user code cannot forge private cache operations, and Resource summary cache keys change when the private cache capability proof changes.

## 2026-06-01 checkpoint

`SourceCapabilityUseSite::PrivateCacheBoundary` を追加し、`PrivateCacheOp` と exact span を `SourceCapabilities` の proof set と source capability policy hash に含めるようにした。

`SourceCapabilityProofFact::PrivateCacheBoundary` と compiler-owned `private_cache_create` / `private_cache_lookup` / `private_cache_insert` / `private_cache_drop` intrinsic collector も追加した。これにより、将来の stdlib memo backend は direct `SourceCapabilities` 構築ではなく既存 proof builder 経由で private cache use-site proof を発行できる。

この checkpoint では `PrivateCache` を pure へ mask する許可はまだ追加していない。trusted stdlib memo backend の actual operation span と Resource IR private cache operation の照合、non-escape proof、user code による forged private cache operation の拒否は残件である。

検証:

- `cargo test -p nepl-core source_capabilit --lib -- --nocapture`
- `cargo test -p nepl-core private_cache --lib -- --nocapture`

残件:

- private cache operation の span と `SourceCapabilityUseSite::PrivateCacheBoundary` の照合。
- shifted/copied source span を Resource summary cache key で stale hit させない regression。
