---
id: ISS-20260522T004417565Z-DROP-CAPABILITY-ACCEPTS-NON-DROP-FIR-5F29D95E
title: "Drop capability accepts non-drop first method as destructor"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/diagnostic_codes.rs, nepl-core/src/typecheck/driver.rs, nepl-core/src/resource/drop_call_identity.rs, nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T004417565Z-DROP-CAPABILITY-ACCEPTS-NON-DROP-FIR-5F29D95E: Drop capability accepts non-drop first method as destructor

## 概要

#capability drop traits without a method named drop are treated as if their first method were the destructor. Resource lowering and auto drop insertion can therefore call or certify an arbitrary method as Drop proof.

## 対象

- `nepl-core/src/diagnostic_codes.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/src/resource/drop_call_identity.rs`
- `nepl-core/src/passes/drop_insertion.rs`
- `nepl-core/tests/drop.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 開発方針: https://zenn.dev/bem130/articles/1b352797de94e7

## 問題

#capability drop traits without a method named drop are treated as if their first method were the destructor. Resource lowering and auto drop insertion can therefore call or certify an arbitrary method as Drop proof.

## 影響

A trait capability declaration with the wrong method shape can make Resource IR emit Drop proof or auto destructor calls for a non-destructor method, weakening memory-safety proof for non-Copy cleanup.

## 修正方針

Require #capability drop traits to expose an explicit drop method. Remove first-method fallback from DropCallIdentityIndex and drop insertion planning, then add regressions for non-drop capability methods.

## 検証

Run focused drop/resource_ir tests plus nepl-core check/fmt and issues check.

## 解決内容

2026-05-22 に Agent 1 が修正した。

- `TypeDiagnosticCode::TraitDropMethodMissing` を追加し、`#capability drop` を持つ trait が `drop` method を持たない場合に型検査で拒否するようにした。
- `DropCallIdentityIndex` から、`drop` method が無い Drop capability trait の先頭 method を destructor とみなす fallback を削除した。
- auto drop insertion の `DropPlan` 構築から同じ fallback を削除し、明示的な `drop` method だけを auto destructor call の対象にした。
- `drop_capability_requires_method_named_drop` を追加し、non-drop method を持つ Drop capability trait が診断コードで拒否されることを固定した。

## 回帰テスト

- `cargo test -p nepl-core --test drop drop_capability_requires_method_named_drop -- --nocapture`
- `cargo test -p nepl-core --test drop drop_capability_parses_and_compiles -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_monomorphized_drop_trait_call_still_emits_drop_proof -- --nocapture`
