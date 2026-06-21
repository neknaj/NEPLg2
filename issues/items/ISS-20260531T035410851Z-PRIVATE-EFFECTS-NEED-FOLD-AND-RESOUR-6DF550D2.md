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

## 2026-06-21 selfhost private effect no-escape gate checkpoint

Selfhost 側に `memo_trait_operation_private_effect_no_escape_gate.nepl` を追加し、`Eq` / `Hash` method body の `PrivateState` / `PrivateCache` effect summary を Resource IR no-escape proof table と照合してから既存 method body fact table へ投入する checker-layer boundary を固定した。

完了したこと:

- final `SelfhostMemoTraitOperationMethodBodyFact` は body root identity を保持しないため、fact table を後から補正せず、HIR root から fact を作る直前に proof を適用する。
- proof key は `SelfhostTypeId`、operation、body module fingerprint、body root、effect、元 escape state の完全一致にし、root mismatch と module fingerprint mismatch を stage0 smoke で確認する。
- `body_module_fingerprint == 0` は proof record と identity-bearing input の両方で拒否し、同一 key の duplicate proof は `ProofDuplicate` にする。
- proof lookup は `PrivateState + NotApplicable` / `PrivateCache + NotApplicable` だけで行い、`Proven -> NoEscapeProven`、`Refuted -> MayEscape`、`Missing` / `Unknown -> NotApplicable` として fail-closed に写す。
- private effect summary が事前に `NoEscapeProven` を持つ場合は `UnexpectedPreProvenNoEscape` で拒否し、この gate より前の bypass を認めない。

subagent review:

- Raman review は、既存 `SelfhostMemoTraitOperationMethodBodyFact` に body identity がないため、最終 fact の後補正ではなく identity-bearing input を gate に通す必要があると指摘した。
- 指摘に従い、proof key と scan input に body module fingerprint / body root を保持し、placeholder fingerprint と duplicate proof を fail-closed にする形にした。

検証:

- `node --check nodesrc/test_selfhost_memo_trait_operation_private_effect_no_escape_gate_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_private_effect_no_escape_gate_contract.js`
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='600000'; node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/check/module/memo_trait_operation_private_effect_no_escape_gate.nepl --dist web/dist -o tmp/selfhost-private-effect-no-escape-gate-doctest.json`。1/1。

残件:

- actual Resource IR proof producer が `PrivateState` / `PrivateCache` の fresh region / non-escape evidence を発行して、この gate の proof table に渡す。
- Resource summary body hash / capability policy hash / artifact policy hash に private effect operation と mask policy version を投影する。
- memo_call backend request-evidence proof と private effect mask を接続し、`RequestEvidenceProven` を backend / effect mask 完了と誤認しない上位 orchestration を追加する。
- sealed backend representation、`.neplobj` / `.neplproof` stable key projection、private cache hit / miss / size / clear / raw identity observation ban を接続する。

## 2026-06-21 selfhost operation impl candidate builder private effect proof path checkpoint

`memo_trait_operation_impl_candidate_builder.nepl` に、caller supplied `SelfhostMemoTraitOperationPrivateEffectNoEscapeProofTable` を受ける proof-aware builder API を追加した。

完了したこと:

- 既存の proof なし builder API は互換のまま残した。
- proof-aware API は method body scan record から fact table を作る直前で `memo_trait_operation_private_effect_no_escape_gate` を通し、body root identity を失う前に private effect proof を照合する。
- `body_module_fingerprint` は builder call 全体の HIR module body identity として扱い、proof key は `SelfhostTypeId`、operation、body module fingerprint、body root、effect、元 escape state の完全一致にした。
- missing proof は accepted `Pure` にせず、`NotApplicable` を保持して operation purity gate 側で `Unknown` 相当に残す。duplicate proof は gate error として fail-closed にした。
- candidate builder は operation evidence、aggregate proof、Resource proof production、memo_call backend bytes、sealed backend representation、artifact keyを作らない。

検証:

- `node --check nodesrc/test_selfhost_memo_trait_operation_impl_candidate_builder_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_impl_candidate_builder_contract.js`
- `node nodesrc/run_source_policy_regressions.js` 相当。ログは最後の `test_zed_extension_no_tracked_target.js` まで到達し、failure pattern は無い。

残件:

- actual Resource IR proof producer が `PrivateState` / `PrivateCache` の fresh region / non-escape proof table を発行して、この proof-aware path へ渡す。
- scanner / upper orchestrator が actual proof source と同じ body module fingerprint を proof-aware public impl materializer へ渡す。
- Resource summary body hash / capability policy hash / artifact policy hash に private effect operation と mask policy version を投影する。
- memo_call backend request-evidence proof と private effect mask を接続する。

## 2026-06-21 selfhost public impl materializer / orchestrator private effect proof transport checkpoint

`memo_trait_operation_public_impl_materializer.nepl` と `memo_trait_public_impl_surface_orchestrator.nepl` に、caller supplied private-effect proof table を public impl materialization へ運ぶ proof-aware entry を追加した。

完了したこと:

- materializer は proof-aware builder を呼ぶ前に、call-level `body_module_fingerprint` が placeholder でないことと、source record table の全 `module_fingerprint` が一致することを検査する。
- scanner-output proof entry、AST-records proof entry、scanner-output generic+proof entry、AST-records generic+proof entry を追加し、generic connector input と proof table は materializer にだけ渡す。
- public surface normalizer / hash composer の順序は既存 entry と同じままにし、proof table を public surface hash authority にしない。
- orchestrator は candidate builder を直接呼ばず、operation materializer の proof-aware API だけを呼ぶ。
- duplicate proof は existing private effect gate / candidate builder の `PrivateEffectNoEscapeGateRejected(ProofDuplicate)` として materializer/orchestrator error に伝播する。
- この checkpoint は Resource IR proof producer、proof store、operation evidence、aggregate proof、memo_call backend bytes、sealed backend representation、artifact key、effect maskを作らない。

検証:

- `node --check nodesrc/test_selfhost_memo_trait_operation_public_impl_materializer_contract.js`
- `node nodesrc/test_selfhost_memo_trait_operation_public_impl_materializer_contract.js`
- `node --check nodesrc/test_selfhost_memo_trait_public_impl_surface_orchestrator_contract.js`
- `node nodesrc/test_selfhost_memo_trait_public_impl_surface_orchestrator_contract.js`
- `node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl --dist web/dist -o tmp/selfhost_materializer_doctest.json -j 1`
- `node nodesrc/run_selfhost_doctest_check.js -i stdlib/neplg2/core/check/module/memo_trait_public_impl_surface_orchestrator.nepl --dist web/dist -o tmp/selfhost_orchestrator_doctest.json -j 1`

残件:

- actual Resource IR proof producer が `PrivateState` / `PrivateCache` の fresh region / non-escape proof table を発行して、この proof-aware path へ渡す。
- Resource summary body hash / capability policy hash / artifact policy hash に private effect operation と mask policy version を投影する。
- memo_call backend request-evidence proof と private effect mask を接続する。
