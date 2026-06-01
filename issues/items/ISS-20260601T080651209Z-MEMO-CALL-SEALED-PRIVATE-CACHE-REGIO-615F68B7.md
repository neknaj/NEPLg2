---
id: ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7
title: "memo_call sealed private cache region proof"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/resource/effect_check.rs; nepl-core/src/effects.rs; nepl-core/src/resource/model.rs; nepl-core/src/typecheck/memo_call.rs"
---

# ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7: memo_call sealed private cache region proof

## 概要

memo_call cannot be folded to Pure through SourceCapability alone; it needs a sealed fresh private cache region and Resource IR non-escape proof.

## 対象

- `nepl-core/src/resource/effect_check.rs; nepl-core/src/effects.rs; nepl-core/src/resource/model.rs; nepl-core/src/typecheck/memo_call.rs`

## 根拠

- `PrivateEffectRegion::UnsealedIntrinsic` は trusted intrinsic 由来の provenance であり、fresh / sealed / non-escaping region の証明ではない。
- `SourceCapabilityUseSite::PrivateCacheBoundary` は exact file / span / operation / region の trusted use-site を証明するが、cache region が外部へ escape しないことは証明しない。
- `PrivateCacheOp::{Create, Lookup, Insert, Drop}` を Pure に mask できるのは、同じ sealed fresh region boundary 内に閉じており、cache internal reference、raw pointer、owner token、stats、clear、hit/miss observation、storage identity が public result や public state へ出ない場合だけである。
- 現行 `memo_call @pure_named_func` Phase 1 は typecheck accepted / rejected matrix を固定しただけで、backend cache representation と sealed region proof はまだ持たない。

## 問題

memo_call cannot be folded to Pure through SourceCapability alone; it needs a sealed fresh private cache region and Resource IR non-escape proof.

## 影響

Without a sealed region proof, memo_call could expose cache storage identity, stats, clear/ref APIs, raw pointers, or stale proof cache entries while still appearing Pure.

## 修正方針

Introduce a sealed compiler-owned private cache region for memo_call, keep UnsealedIntrinsic unmasked, prove non-escape separately from SourceCapability exact use-site proof, and only then mask PrivateCache operations to Pure.

## 受け入れ条件

- `UnsealedIntrinsic` の `PrivateCache` は exact SourceCapability があっても pure function 内で拒否される。
- sealed region は compiler-known `memo_call` backend だけが発行できる fresh private cache region として表現する。
- SourceCapability exact proof と Resource IR non-escape proof は別 authority として扱う。
- same operation / same span でも region が違えば `PrivateCacheOutsideBoundary` を suppress しない。
- sealed region 由来の reference、raw pointer、owner token、storage identity、stats、clear、hit/miss observation、cache region id、function wrapper identity を public result、public field、global、impure / unknown call に出さない。
- proof 済み sealed region の `PrivateCacheOp::{Create, Lookup, Insert, Drop}` だけを Pure mask 候補にする。
- accepted path は既存の `memo_call @pure_named_func` 制約を維持する。`memo_call @func arg` immediate application、function literal、function value alias / pass-through は sealed backend が境界を保持できるまで拒否する。

## 関連 issue

- `ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4`
- `ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF`
- `ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2`
- `ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7`
- `ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C`

## 検証

Negative Resource IR tests must reject unsealed PrivateCache in pure functions, region mismatch, cache reference/raw pointer/owner token escape, stats/clear observation, and immediate memo_call paths before sealed proof exists.

最小 regression:

- `UnsealedIntrinsic` の `PrivateCache` は exact SourceCapability があっても `PrivateCacheInPureFunction` のまま拒否する。
- sealed proof がない `PrivateCache` は `PrivateCacheInPureFunction` のまま拒否する。
- same file / span / operation でも region mismatch なら `PrivateCacheOutsideBoundary` を拒否する。
- sealed region 由来の raw pointer / reference / owner token を return する Resource IR fixture を拒否する。
- cache stats / clear / ref のような観測 API を mask 対象外として拒否する。
- `memo_call @pure_named_func` の accepted matrix を維持し、即時適用や unresolved / impure / capturing / generic function value は拒否する。

## 2026-06-01 region exactness checkpoint

`PrivateEffectRegionId` と `PrivateEffectRegion::SealedCompilerPrivateCache(id)` を追加し、sealed private cache region を `UnsealedIntrinsic` と区別できるようにした。

この checkpoint は sealed region を発行する memo backend や Pure mask accepted path ではない。目的は、将来 sealed region proof を導入したときに SourceCapability と Resource summary cache が region identity を潰さないよう、先に fail-closed 境界を固定することである。

実装した境界:

- SourceCapability policy hash は private cache region の variant と numeric id を hash する。
- Resource summary body hash は `PrivateState` / `PrivateCache` effect の region variant と numeric id を hash する。
- Resource function body hash namespace は `neplg2-resource-function-body-v4` に上げた。
- Resource effect boundary gate は same file / same span / same operation でも region mismatch を拒否する。
- `UnsealedIntrinsic` capability は `SealedCompilerPrivateCache(id)` の diagnostic を許可せず、sealed capability も別 id や `UnsealedIntrinsic` を許可しない。

追加 regression:

- `resource_effect_gate_rejects_private_cache_region_mismatch`
- `resource_function_body_hash_tracks_private_cache_region_identity`
- SourceCapability policy hash が sealed region id の差を追跡する regression

残件:

- compiler-known `memo_call` backend が sealed fresh region を発行する実装。
- sealed region の cache storage、reference、raw pointer、owner token、stats、clear、hit/miss observation が外部へ escape しない Resource IR proof。
- proof 済み sealed region のみを `PrivateCacheInPureFunction` から Pure mask 候補へ進める fold/checker 実装。

## 2026-06-01 mask proof index scaffold checkpoint

`PrivateCacheMaskProofIndex` を追加し、Resource effect checker が `PrivateCacheInPureFunction` を
抑制するための authority を SourceCapability から分離した。

この checkpoint でも通常 compile path は空の proof index だけを使う。したがって、
`UnsealedIntrinsic` の `PrivateCache` や sealed proof がない `PrivateCache` は従来通り
`PrivateCacheInPureFunction` として fail-closed に拒否される。

実装した境界:

- mask proof は function name、sealed private cache region、operation の完全一致を要求する。
- `UnsealedIntrinsic` は proof entry があっても mask しない。
- proof が一致しても `PrivateCacheOutsideBoundary` は残す。SourceCapability exact use-site proof と
  Resource IR non-escape proof は別 authority のままである。
- Resource effect checker の summary helper 経路も空 proof index を渡し、既存 summary / provenance
  計算で private cache を暗黙に pure 扱いしない。

この checkpoint は non-escape proof の保存先と、sealed private cache handle が戻り値へ
escape しないことを検査する最初の taint propagation を追加する。まだ proof を自動発行して
accepted memoization path へ接続する段階ではない。

追加 regression:

- `private_cache_mask_proof_requires_exact_sealed_region_and_operation`
- `private_cache_mask_proof_never_allows_unsealed_intrinsic_region`
- `private_cache_mask_proof_suppresses_only_pure_context_diagnostic`
- `private_cache_mask_proof_rejects_unproven_region`
- `taint_table_tracks_exact_sealed_region_through_copy_and_move`
- `taint_table_ignores_unsealed_intrinsic_region`
- `aggregate_taint_is_visible_from_the_whole_output_place`
- `sealed_private_cache_create_output_cannot_escape_through_return`
- `typed_effect_check_tracks_private_cache_taint_without_checked_mem_ptr_access`

追加検証:

- `cargo test -p nepl-core private_cache --lib -- --nocapture`
- `cargo test -p nepl-core private_cache_mask --lib -- --nocapture`
- `cargo test -p nepl-core private_cache_effect_boundary --lib -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`

残件:

- tainted cache handle / reference / raw pointer / owner token を impure call、unknown call、
  public field、global state へ渡す escape 診断。
- `PrivateCacheMaskProofIndex` を手動 test fixture ではなく、Resource IR の non-escape proof
  生成結果から作る実装。
- `memo_call` backend が sealed fresh region と private cache operation を発行する実装。
- lookup result が cache 内部参照ではなく owned / copied value であることの proof。

## 2026-06-02 `.neplobj` direct-call boundary review

`.neplobj` direct-call fragment の same-session reuse は、`FnValue`、`CallIndirect`、
`MemoizedFunctionValue`、`memo_call` を解決しない境界を維持している。direct-call fragment は
materialized callable の raw body を wasm backend へ渡す artifact であり、function identity、
closure identity、PrivateCache region、non-escape proof、cache lookup result ownership を証明する
artifact ではない。

subagent review でも、中間ファイル再利用によって `memo_call` / private cache pure check が直ちに
壊れる経路は見つからなかった。現在の通常 compile path は空の `PrivateCacheMaskProofIndex` を使うため、
sealed proof がない `PrivateCache` は `PrivateCacheInPureFunction` として fail-closed に残る。

今後 `.neplproof` に private cache mask proof を保存する場合は、function identity、sealed region、
operation、source capability policy、private effect policy hash、non-escape proof boundary が一致しない
artifact を payload decode 前または proof merge 前に拒否する必要がある。
