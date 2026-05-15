---
id: ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D
title: "Owner aggregate constructor capability is file-wide instead of constructor-specific"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/effect_check.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260515T020307026Z-OWNER-AGGREGATE-CONSTRUCTOR-CAPABILI-91ECE78D: Owner aggregate constructor capability is file-wide instead of constructor-specific

## 概要

OwnerAggregateConstructorBoundary is granted to the whole compiler-owned stdlib file when the source contains any unqualified constructor-like symbol. That allows unrelated constructor evidence such as Diag to authorize owner-backed aggregate constructors such as Vec in the same file.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/effect_check.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- `OwnerAggregateConstructorBoundary` は owner-backed aggregate の direct constructor を通常 source から隔離するための capability であり、unrelated constructor syntax から過大付与されてはいけない。
- 修正前は source capability が file-wide bool だったため、compiler-owned stdlib source 内の `Diag` constructor evidence と `Vec` / `HashMap` などの owner-backed constructor authority が区別されていなかった。

## 問題

OwnerAggregateConstructorBoundary is granted to the whole compiler-owned stdlib file when the source contains any unqualified constructor-like symbol. That allows unrelated constructor evidence such as Diag to authorize owner-backed aggregate constructors such as Vec in the same file.

## 影響

The source proof for owner-backed aggregate construction is broader than the actual constructor being authorized, weakening the Stage 6 memory-safety boundary and allowing future stdlib code to gain owner aggregate construction authority from unrelated syntax.

## 修正方針

Store constructor evidence by constructor name in SourceCapabilities and require the typecheck constructor gate to match the specific owner-backed aggregate constructor being applied.

## 検証

Add loader regressions for unrelated constructor evidence, update static-check source policy, and run focused nepl-core source capability tests plus issue validation.

## 解決内容

`SourceCapability::OwnerAggregateConstructorBoundary` を `OwnerAggregateConstructorBoundary(String)` に変更し、constructor evidence を名前ごとに保持するようにした。loader は parsed source から unqualified constructor-like symbol を収集し、`Diag` evidence なら `Diag` だけ、`Vec` evidence なら `Vec` だけを capability として登録する。

typecheck 側では owner-backed aggregate constructor gate が `span` だけでなく実際に適用中の constructor 名も渡すようにした。これにより、`SourceCapabilities::owner_aggregate_constructor_boundary("Diag")` を持つ source でも `VecBox` / `Vec` のような別 constructor は `type.owner_aggregate.constructor_restricted` のまま拒否される。

回帰として loader unit test に unrelated constructor evidence regression を追加し、`resource_ir.rs` に unrelated constructor capability では owner-backed constructor を通せない compile failure を追加した。`nodesrc/test_static_check_boundary_responsibility.js` も、constructor evidence が `BTreeSet<String>` と enum で管理されることを確認するよう更新した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`: 5 passed
- `cargo test -p nepl-core typecheck_rejects_owner_backed_constructor_with_unrelated_constructor_capability --test resource_ir -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
