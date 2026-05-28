---
id: ISS-20260528T123956177Z-RESOURCE-SUMMARY-RAW-INIT-DEPENDENCY-70A9D8F6
title: "Resource summary raw-init dependency closure has unkeyable dependencies"
area: core
status: open
resolved: false
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
