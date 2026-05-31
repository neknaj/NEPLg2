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
- Resource IR の `PrivateCache` operation に fresh region identity / mask boundary provenance を持たせる。
- `SourceCapability` exact span は trusted use-site の証明に留め、region non-escape proof が入るまで `PrivateCacheInPureFunction` は fail-closed のまま維持する。

## 2026-06-01 exact boundary gate checkpoint

`ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary` を追加し、Resource effect boundary gate で `SourceMap::private_cache_boundary_allowed_at(span, operation)` を使うようにした。これにより、Resource IR の `EffectOp::PrivateCache` は trusted SourceCapability proof と同一 file / exact span / same operation の場合だけ boundary 診断を suppress できる。

この checkpoint でも `PrivateCacheInPureFunction` は SourceCapability で suppress しない。SourceCapability は trusted use-site かどうかの証明であり、private cache effect を Pure へ mask する authority ではない。fresh region、non-escape、cache ref/stats/clear 非露出の Resource IR proof が入るまで、pure function 内の `PrivateCache` は fail-closed のまま維持する。

追加 regression:

- exact operation / exact span / exact file の private cache boundary だけを許可する。
- same span でも `Lookup` proof で `Insert` operation は通さない。
- same operation でも shifted span や別 file span は通さない。
- private cache capability があっても `PrivateCacheInPureFunction` は通さない。

検証:

- `cargo test -p nepl-core private_cache --lib -- --nocapture`
- `cargo test -p nepl-core resource_effect_gate --lib -- --nocapture`

残件:

- Resource IR private cache operation の actual span を stdlib memo backend の proof span と一致させる integration regression。
- `PrivateCache` operation に fresh region identity / mask boundary provenance を持たせる。
- cache lookup result が owned/copy/clone value であり、cache internal reference、stats、clear、raw identity が外へ出ないことを Resource IR で証明する。

## 2026-06-01 actual span integration checkpoint

`private_cache_*` intrinsic 名を `PrivateCacheOp` へ変換する shared helper を `effects.rs` に置き、SourceCapability collector と Resource IR effect lowering が同じ primitive identity を参照するようにした。`intrinsic_internal_effect("private_cache_lookup")` は `InternalEffect::PrivateCache { operation: Lookup }` を返し、Resource IR lowering は `ResourceOp::CallEffect { effect: EffectOp::PrivateCache { .. }, span: expr.span }` を出す。

SourceCapability 側の private cache proof span は intrinsic 名 token ではなく intrinsic expression 全体の span に変更した。これは Resource IR effect boundary gate が `CallEffect` の `expr.span` を照合するためであり、trusted use-site の authority を同じ構文単位へ揃えるためである。あわせて、空引数 intrinsic の parser span が `)` まで含むように修正した。

追加 regression:

- `private_cache_*` intrinsic は内部 effect として `PrivateCache` に分類され、unmasked surface は `Impure` になる。
- Resource IR lowering は private cache intrinsic から `EffectOp::PrivateCache` を expression span で出す。
- configured stdlib source の private cache capability は expression span でだけ許可され、intrinsic-name token span では許可されない。

検証:

- `cargo test -p nepl-core private_cache --lib -- --nocapture`
- `cargo test -p nepl-core private_cache_intrinsic_lowers_call_effect_at_expression_span --lib -- --nocapture`

残件:

- `private_cache_*` intrinsic の typecheck signature はまだ public surface として固定しない。Phase 1 backend の sealed representation と region proof に合わせて stdlib memo backend integration test で扱う。
- `PrivateCacheInPureFunction` は引き続き fail-closed。fresh region / non-escape proof が入るまで SourceCapability だけでは Pure mask しない。

## 2026-06-01 region provenance checkpoint

`SourceCapabilityUseSite::PrivateCacheBoundary` と `SourceCapabilityProofFact::PrivateCacheBoundary` に `PrivateEffectRegion` を追加した。`private_cache_*` intrinsic collector は現時点では `PrivateEffectRegion::UnsealedIntrinsic` を発行する。

Resource effect boundary gate は `PrivateCacheOutsideBoundary` について exact file / exact span / same operation / same region の SourceCapability proof だけで suppress する。`PrivateCacheInPureFunction` は引き続き suppress しない。

source capability policy hash は private cache operation と span に加えて region provenance も hash する。これにより、将来 `UnsealedIntrinsic` と fresh sealed region の boundary が分かれたときに、古い proof artifact が同じ capability policy として stale hit しない。

追加検証:

- `cargo test -p nepl-core source_capabilit --lib -- --nocapture`
- `cargo test -p nepl-core resource_effect_gate --lib -- --nocapture`
- `cargo test -p nepl-core private_cache --lib -- --nocapture`

残件:

- `PrivateCacheBoundary` が trusted use-site であることと、fresh region の non-escape proof を分離したまま stdlib memo backend へ接続する。
- region mismatch を実際に持てる sealed backend region 導入後、same operation / same span でも別 region proof は通さない regression を追加する。
