---
id: ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9
title: "Type ascription does not consistently enter expected-check mode"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_call_apply.rs, nepl-core/src/typecheck"
---

# ISS-20260520T052151589Z-TYPE-ASCRIPTION-DOES-NOT-CONSISTENTL-BFF974A9: Type ascription does not consistently enter expected-check mode

## 概要

Prefix type annotations are parsed and applied, but the checker still relies on post-inference unification in several paths. Fully annotated expressions should be checked against the expected type without unnecessary overload or generic search.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/trait_call_apply.rs, nepl-core/src/typecheck`

## 根拠

- `plan.md` は `<T>` 形式の型注釈を続く式への前置注釈として定義している。
- `nepl-core/src/parser.rs` / `nepl-core/src/ast.rs` には `PrefixItem::TypeAnnotation` があり、構文上は任意式の前置型注釈を扱えている。
- `nepl-core/src/typecheck/prefix_check.rs` は `pending_ascription` により型注釈を保持するが、tuple 状態であり、推論モードと期待型に対する検査モードの責務分割が型で明示されていない。
- `nepl-core/src/typecheck/overload_selection.rs`、`trait_call_apply.rs` には `expected_ret` が渡る経路があるが、expected type constraint と必要探索の境界が設計として固定されていない。
- 設計方針として、型安全・静的検査・enum/match による網羅性・技術的負債を残さないことが必須である。

## 問題

Prefix type annotations are parsed and applied, but the checker still relies on post-inference unification in several paths. Fully annotated expressions should be checked against the expected type without unnecessary overload or generic search.

## 影響

Compile time can grow from avoidable candidate exploration, and the typechecker design remains unclear about when it is inferring versus verifying an annotated expression.

## 修正方針

Introduce an explicit expected-check mode for type ascription boundaries while preserving inference when no annotation is present. Use expected types to prune overload, trait, and generic resolution before expensive exploration.

設計書: [NEPLg2 型注釈 expected-check 設計計画](../../doc/neplg2/type_ascription_expected_check_plan.md)

型注釈は探索を完全に停止する印ではない。generic type parameter、trait application、receiver type、overload argument の制約が不足している場合は、必要十分な探索を行う。ただし探索は無制限な総当たりにせず、expected type、argument constraints、effect context、trait bounds を typed constraint として集約し、候補を採用または拒否した根拠を enum / struct で保持する。

`<T>` が十分な情報を与える場合は、その式を `Check(T)` として処理し、不要な候補展開を避ける。`<T>` だけでは不十分な場合は、注釈なし推論へ無秩序に戻るのではなく、不足している制約だけを探索し、制約が閉じても一意に決まらなければ曖昧性または注釈不足として診断する。

## 検証

Add focused compiler regressions and source-policy tests proving annotated expressions select/check directly, unannotated expressions still infer, and no string allowlist replaces typed expected-type evidence.

追加する regression は少なくとも次を含める。

- 注釈なしでは overload ambiguous になる式が、期待戻り値注釈で一意に決まる。
- 注釈があっても generic type parameter が不足する式では、引数型や trait bound から必要探索して解ける。
- 注釈があっても制約が閉じた後に複数候補が残る式では、無理に選ばず曖昧性診断を出す。
- 注釈あり経路でも stdlib 関数名 allowlist や文字列 key に依存しない。
