---
id: ISS-20260506T201433509Z-RESOURCE-CONDITION-FACTS-DROP-NONZER-5EE6B7A6
title: "Resource condition facts drop nonzero relational guards"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/lower_condition.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260506T201433509Z-RESOURCE-CONDITION-FACTS-DROP-NONZER-5EE6B7A6: Resource condition facts drop nonzero relational guards

## 概要

Resource IR lowering only preserves zero/one comparisons as condition facts. Guards such as lt i len are dropped, so later initialized range summaries cannot prove that a dynamic raw-memory offset is bounded by a returned length field.

## 対象

- `nepl-core/src/resource/lower_condition.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceConditionFact` は zero/one 比較だけを `EqZero` / `Positive` などの単項 fact にしており、`lt i len` のような length guard は `None` になっていた。
- `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の returned raw header range summary では、caller 側の `i < len` を typed fact として保持できなければ、dynamic offset read を安全に通せない。
- この段階で source shape や文字列に依存した guard 推測へ進むと、静的検査の網羅性と保守性を落とすため、Resource IR の enum に relation fact を追加する必要があった。

## 問題

Resource IR lowering only preserves zero/one comparisons as condition facts. Guards such as lt i len are dropped, so later initialized range summaries cannot prove that a dynamic raw-memory offset is bounded by a returned length field.

## 影響

Length-guarded raw memory reads either remain rejected even when the source has an explicit guard, or future range-summary work would have to rely on stringly/source-shape checks instead of typed Resource IR facts. That weakens the static-check design and blocks the parent returned-range summary issue.

## 修正方針

Represent nonzero i32 comparisons as a typed ResourceConditionFact relation with an enum relation operator. Keep existing zero-value facts for owner/realloc refinement, but preserve general lt/le/gt/ge/eq/ne guards for later range summaries and dump them in Resource IR snapshots.

## 検証

Add a Resource IR regression that lowers lt i len into the typed relation fact, run focused nepl-core resource tests, and keep source policy/issue checks passing.

## 2026-05-07 修正

`ResourceConditionFact::I32Relation` と `ResourceI32RelationOp` を追加し、`lt` / `le` / `gt` / `ge` / 非 zero の `eq` / `ne` を typed relation fact として保持するようにした。

既存の zero-value fact は owner / realloc / variant summary refinement に使われているため維持した。`lt 0 x` や `le x 0` のような単項 condition は従来通り `Positive` / `NonPositive` へ下げ、`lt i len` のような relation だけを `I32Relation { left, op, right }` にする。各 consumer は exhaustive `match` で relation fact を明示的に扱い、まだ scalar refinement へ使わない箇所では意図して無視する。

Resource IR dump も `fact=i32_relation(%i < %len)` 形式で relation fact を表示するため、今後の range summary regression で typed guard が存在することを確認できる。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_nonzero_i32_relation_condition_fact -- --nocapture`: passed

## 関連

- [ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38](./ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38.md)
