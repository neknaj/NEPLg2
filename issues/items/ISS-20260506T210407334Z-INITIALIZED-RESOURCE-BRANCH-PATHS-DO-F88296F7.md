---
id: ISS-20260506T210407334Z-INITIALIZED-RESOURCE-BRANCH-PATHS-DO-F88296F7
title: "Initialized resource branch paths do not record typed condition facts"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/condition_fact.rs, nepl-core/src/resource/initialized_alias*.rs"
---

# ISS-20260506T210407334Z-INITIALIZED-RESOURCE-BRANCH-PATHS-DO-F88296F7: Initialized resource branch paths do not record typed condition facts

## 概要

The initialized Resource checker clones branch path state but does not record ResourceConditionFact into RawCellAddressAliases. Owner checking records those facts, so typed i32 relation proofs exist in one checker path but are invisible to initialized cell availability and future guarded range summaries.

## 対象

- `nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/condition_fact.rs, nepl-core/src/resource/initialized_alias*.rs`

## 根拠

- `owner_control.rs` は branch path へ入る前に `record_condition_fact_value_constraints` を呼び、truthy / false の条件 fact を raw alias state へ反映している。
- `initialized_control.rs` は同じ `RawCellAddressAliases` を使っているにもかかわらず、realloc condition だけを処理しており、`ResourceConditionFact::I32Relation` を initialized cell check 側へ伝播していなかった。
- guarded initialized range summary は initialized checker の `ensure_available` / raw load path から参照されるため、owner checker だけに relation fact が残っても不十分である。

## 問題

The initialized Resource checker clones branch path state but does not record ResourceConditionFact into RawCellAddressAliases. Owner checking records those facts, so typed i32 relation proofs exist in one checker path but are invisible to initialized cell availability and future guarded range summaries.

## 影響

Guarded dynamic raw loads cannot be proven by initialized checking even after ResourceConditionFact::I32Relation and I32RelationFacts exist. Leaving this gap would force the range checker to re-read HIR conditions or weaken memory safety.

## 修正方針

Record typed condition facts in initialized branch paths before checking then/else ops, using the existing condition_fact API so truthy and false branches share the same enum-based relation handling as owner checking.

## 検証

Add focused regression coverage for initialized branch fact recording behavior, run nepl-core relation tests, resource responsibility checks, issue checks, and cargo check.

## 2026-05-07 対応結果

`initialized_control.rs` に initialized checker 用の branch condition fact application を追加した。then / else の path state を clone した直後に `record_condition_fact_value_constraints` を実行し、その後に既存の realloc condition handling を行うため、typed i32 relation と realloc success/failure の両方が同じ branch state に反映される。

この修正は condition fact の解釈を duplicated logic にせず、owner checker と同じ `condition_fact.rs` の enum-based implementation を使う。truthy branch は元 relation、false branch は negated relation として記録され、将来の initialized range summary が `RawCellAddressAliases::i32_relation_truth` を直接参照できる。

回帰として initialized checker 側の truthy / false branch relation recording unit test を追加した。
