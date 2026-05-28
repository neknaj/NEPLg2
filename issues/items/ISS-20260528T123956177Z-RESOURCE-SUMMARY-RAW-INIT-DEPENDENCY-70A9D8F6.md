---
id: ISS-20260528T123956177Z-RESOURCE-SUMMARY-RAW-INIT-DEPENDENCY-70A9D8F6
title: "Resource summary raw-init dependency closure has unkeyable dependencies"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/resource/resource_summary_value_cache/dependency_hash.rs; nepl-core/src/resource/resource_summary_value_cache/body_hash.rs"
---

# ISS-20260528T123956177Z-RESOURCE-SUMMARY-RAW-INIT-DEPENDENCY-70A9D8F6: Resource summary raw-init dependency closure has unkeyable dependencies

## 概要

RPN same-session code edit after dependency closure support still reports raw_init_param_facts_unstable_key_bypasses=176. Dependency-bearing raw-init facts can now be safely keyed, but many dependency closure functions still fail body/source/type-boundary key construction.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/dependency_hash.rs; nepl-core/src/resource/resource_summary_value_cache/body_hash.rs`

## 根拠

- `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` の verified 測定で、dependency closure support 後も `raw_init_param_facts_unstable_key_bypasses=176` が残った。
- 同測定では `raw_init_param_facts_stores=2` / `hits=2` まで進んでいるため、raw-init cache の基本 store/replay 経路ではなく、依存先 closure の keyability が次の支配的 blocker である。
- 現状の統計では `unstable_key` の内訳が、raw body/source body hash 不足、dependency function body hash 不可、type boundary 不可、source policy 不足のどれかまで分かれていない。

## 問題

RPN same-session code edit after dependency closure support still reports raw_init_param_facts_unstable_key_bypasses=176. Dependency-bearing raw-init facts can now be safely keyed, but many dependency closure functions still fail body/source/type-boundary key construction.

## 影響

The cache now stores and replays two raw-init facts, but most dependency-bearing stdlib raw-init facts still recompute on every code edit, keeping Web playground compile time around 8 seconds.

## 修正方針

Split dependency-closure hash failures by missing source policy, raw body/source body hash, unstable type boundary, and body hash unsupported operations; then add stable inputs instead of bypassing broad classes.

## 検証

RPN same-session code edit shows raw_init_param_facts_unstable_key_bypasses decreasing and stores/hits increasing without stale hit regressions.

## 対応結果

2026-05-28 の raw body dependency key checkpoint で verified。

- `ResourceTerminator::RawBody` は source body text を `ResourceFunction` へ直接保持しないため、body hash では backend kind を固定し、必ず source capability policy hash と組み合わせる契約にした。
- `source_capability_policy_hash_for_function` は source path、source content hash、raw body capability use-site を含むため、raw body text や raw memory boundary の変更で dependency closure hash が変わる。
- dependency closure hash failure を dependency graph / identity / body hash / source policy / type boundary の counter へ分割した。
- RPN same-session code edit 測定では `raw_init_param_facts_unstable_key_bypasses` が `176 -> 0` になり、2 回目 compile で `raw_init_param_facts_hits=2` / `resource_summary_value_replay_hits=2` を維持した。

残件は `raw_init_param_facts_unstable_entry_bypasses=119` と `raw_init_param_facts_reprojection_bypasses=67` へ移ったため、`ISS-20260528T125932150Z-RESOURCE-SUMMARY-RAW-INIT-STABLE-ENT-AE09D7D6` と `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E` で扱う。
