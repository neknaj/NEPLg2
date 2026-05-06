---
id: ISS-20260506T203942617Z-RESOURCE-BRANCH-PATHS-DO-NOT-RETAIN--4242E13D
title: "Resource branch paths do not retain typed i32 relation facts"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/condition_fact.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_relation*.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260506T203942617Z-RESOURCE-BRANCH-PATHS-DO-NOT-RETAIN--4242E13D: Resource branch paths do not retain typed i32 relation facts

## 概要

ResourceConditionFact::I32Relation is lowered for guards such as i < len, but condition_fact.rs deliberately ignores it when recording branch-path facts. Later initialized range checks therefore cannot query whether a symbolic offset is proven below a length bound.

## 対象

- `nepl-core/src/resource/condition_fact.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/initialized_alias_relation*.rs`

## 根拠

- `ResourceConditionFact::I32Relation` は lowering / dump には残るが、`record_condition_fact_value_constraints` では明示的に無視されていた。
- 既存の `I32AliasFacts` は literal value と unary condition だけを保持しており、`i < len` / `i >= len` のような二項関係を branch path state として問い合わせる API がなかった。
- initialized range summary は `ResourceOffset::Symbolic` と guard relation を同じ Resource IR state から照合する必要があるため、HIR 条件式を後から再走査する設計にはできない。

## 問題

ResourceConditionFact::I32Relation is lowered for guards such as i < len, but condition_fact.rs deliberately ignores it when recording branch-path facts. Later initialized range checks therefore cannot query whether a symbolic offset is proven below a length bound.

## 影響

The dependent initialized range model would have to re-read branch syntax instead of using Resource IR state, or would be forced to accept/reject dynamic raw loads without a typed proof store.

## 修正方針

Add a typed i32 relation fact store beside existing unary i32 value conditions, record truthy and false branch relations with explicit op negation, preserve facts through alias copy/merge/clear, and add regression coverage for true and false branch queries.

## 検証

Run focused Rust unit tests for relation fact recording, relation copy/merge, resource IR relation lowering regression, cargo check for nepl-core tests, and issue/source policy checks.

## 2026-05-07 対応結果

`I32RelationFacts` を `I32AliasFacts` から分離した typed relation fact store として追加した。value / unary condition は `I32AliasFacts` に残し、`I32RelationFact { left, op, right }` は `initialized_alias_relation*.rs` 側で copy、prefix replacement、clear、path merge の対象にする。

`record_condition_fact_value_constraints` は `ResourceConditionFact::I32Relation` を無視せず、truthy branch ではそのまま、false branch では `Eq <-> Ne` / `Lt <-> Ge` / `Le <-> Gt` / `Gt <-> Le` のように negated op として保存する。query 側は reversed relation も扱い、`i < len` から `len > i` を導ける。

この変更はまだ raw load を許可するものではない。range summary 本体が参照できる typed proof store を整備した段階であり、`i32_relation_truth` が矛盾する fact を検出した場合は `None` にして安全側へ倒す。

回帰として true branch / false branch の relation recording unit test に加え、alias copy と path merge で relation fact が過剰に残らないことを確認する unit test を追加した。
