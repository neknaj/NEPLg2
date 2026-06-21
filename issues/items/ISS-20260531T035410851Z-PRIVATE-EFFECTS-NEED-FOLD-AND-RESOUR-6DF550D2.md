---
id: ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2
title: "Private effects need fold and Resource summary hash integration"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-21
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

## 2026-06-01 sealed memo cache proof dependency

`memo_call` 向けの sealed private cache region proof は
`ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7` に分離した。

Resource summary body hash / private effect policy hash には、sealed region policy version、
region kind、region provenance、private cache operation を含める。session-local region id そのものは
長寿命 cache key にしない。`UnsealedIntrinsic` と sealed fresh region が同じ key へ落ちないことを、
proof artifact 永続化前の受け入れ条件にする。

## 2026-06-21 selfhost private effect boundary model checkpoint

Selfhost 側で `SelfhostEffectKind::{PrivateState, PrivateCache}` を追加し、pure / impure effect solver、method body effect summary、operation purity gate、Drop no-escape gate の private effect contract を固定した。

完了したこと:

- `PrivateState` / `PrivateCache` は `NoEscapeProven` がある場合だけ operation purity gate で Pure 相当へ畳む。
- `NotApplicable` / `MayEscape` は `PrivateEffectEscapeNotProven` で fail-closed にし、missing proof を通常の observable impure error と混ぜない。
- method body summary は `InternalAlloc` / `PrivateState` / `PrivateCache` だけ escape state を保持し、`UnsafeMemory` / `ExternalIo` / `Nondet` は observable effect として escape state を `NotApplicable` に戻す。
- Drop no-escape gate は private effect を `InternalAlloc` proof として合成せず、private effect fact を purity gate へ pass-through する。
- Resource graph input scanner / traversal collector / no-escape producer / materializer は private effect の enum coverage と error payload equality だけを追加し、proof production は引き続き `InternalAlloc + NotApplicable` だけに限定した。
- public function signature hash は既存 effect code を維持し、`PrivateState = 332006`、`PrivateCache = 332007` を append-only code として追加した。`UnsafeMemory = 332003`、`ExternalIo = 332004`、`Nondet = 332005` は移動しない。

subagent review:

- Halley review は blocking 指摘として、public function signature の existing effect stable code をずらしている問題を検出した。
- 指摘を反映し、schema version 1 のまま既存 hash を silently に変えない append-only mapping へ修正し、contract test もその不変条件を固定するように更新した。

検証:

- `node nodesrc/test_selfhost_memo_trait_operation_purity_gate_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_method_body_effect_checker_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_no_escape_gate_contract.js`
- `node nodesrc/test_selfhost_memo_trait_public_function_signature_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_producer_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_materializer_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_traversal_collector_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_resource_graph_input_scanner_contract.js`
- `node nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_drop_impl_fact_table_builder_contract.js`
- `NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/ty/effect.nepl -i stdlib/neplg2/core/check/module/memo_trait_operation_method_body_effect_checker.nepl -i stdlib/neplg2/core/check/module/memo_trait_operation_purity_gate.nepl -i stdlib/neplg2/core/check/module/memo_trait_operation_drop_no_escape_gate.nepl -i stdlib/neplg2/core/proof/solver/effect.nepl -i stdlib/neplg2/core/check/module/memo_trait_public_function_signature.nepl --dist web/dist -o tmp/private-effects-selfhost-doctests.json`。5/5。

残件:

- actual Resource IR proof producer から `PrivateState` / `PrivateCache` の fresh region / non-escape evidence を発行して、今回の selfhost private effect gate へ接続する。
- Resource summary body hash / capability policy hash / artifact policy hash に private effect operation と mask policy version を含める。
- memo_call backend request-evidence proof と private effect mask を接続する。
- sealed backend representation、`.neplobj` / `.neplproof` stable key projection、private cache observation ban を接続する。
