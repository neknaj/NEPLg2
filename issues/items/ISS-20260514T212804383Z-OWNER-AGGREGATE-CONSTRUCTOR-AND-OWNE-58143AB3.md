---
id: ISS-20260514T212804383Z-OWNER-AGGREGATE-CONSTRUCTOR-AND-OWNE-58143AB3
title: "Owner aggregate constructor and owner field projection share one source capability"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/field_access.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260514T212804383Z-OWNER-AGGREGATE-CONSTRUCTOR-AND-OWNE-58143AB3: Owner aggregate constructor and owner field projection share one source capability

## 概要

OwnerAggregateBoundary is a single file-wide capability for two different privileged operations: constructing owner-backed aggregates and projecting owner-token fields out of aggregates. A source file that only has field accessor evidence can also construct owner-backed aggregates, and a file that only has aggregate constructor evidence can also project owner tokens.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/field_access.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- owner-backed aggregate の direct constructor と、aggregate からの owner token field projection は別の特権操作である。
- source capability は「source にある証拠」と「許可される操作」が一対一に近い粒度で対応している必要がある。

## 問題

OwnerAggregateBoundary is a single file-wide capability for two different privileged operations: constructing owner-backed aggregates and projecting owner-token fields out of aggregates. A source file that only has field accessor evidence can also construct owner-backed aggregates, and a file that only has aggregate constructor evidence can also project owner tokens.

## 影響

The static proof is coarser than the authority being granted. This keeps owner-token extraction and owner-backed aggregate construction coupled, making it easier for future stdlib code to acquire more privilege than its source evidence justifies.

## 修正方針

Split the capability into constructor and field-projection variants, let the owner_aggregate source walker collect both evidence kinds separately, check constructors against the constructor capability and owner-token field projection against the field capability, and update focused regressions and source policy.

## 検証

Run focused source capability unit tests, source_map capability tests, static-check boundary policy, issue validation, and diff whitespace checks.

## 解決内容

`SourceCapability::OwnerAggregateBoundary` を削除し、`OwnerAggregateConstructorBoundary` と `OwnerAggregateFieldBoundary` に分離した。`source_capability/owner_aggregate.rs` は constructor-like evidence と field accessor evidence を別々に問い合わせる API へ変更し、loader は検出された evidence kind に対応する capability だけを source file へ付与する。

typecheck 側では、owner-backed aggregate constructor は `owner_aggregate_constructor_boundary_allowed`、owner token field projection は `owner_aggregate_field_boundary_allowed` を見るように分けた。これにより、`field::get` だけを使う source が direct owner aggregate constructor まで許される経路と、constructor だけを使う source が owner token projection まで許される経路を閉じた。

loader regression は、field accessor evidence が constructor capability を付与しないこと、constructor evidence が field projection capability を付与しないことを明示的に検査する形へ更新した。

## 関連

- Parent: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- Doc: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
