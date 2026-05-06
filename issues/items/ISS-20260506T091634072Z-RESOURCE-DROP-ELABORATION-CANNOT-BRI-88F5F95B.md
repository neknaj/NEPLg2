---
id: ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B
title: "Resource drop elaboration cannot bridge monomorphized functions to source HIR"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/hir.rs, nepl-core/src/typecheck/function_check.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/drop_elaboration.rs"
---

# ISS-20260506T091634072Z-RESOURCE-DROP-ELABORATION-CANNOT-BRI-88F5F95B: Resource drop elaboration cannot bridge monomorphized functions to source HIR

## 概要

ResourceDropElaborationPlan は monomorphize 後の Resource IR から構築される。一方で、残る HIR drop insertion の置換作業では source HIR 側の関数単位へ drop plan を戻す必要がある。generic specialization 後の関数名は mangled symbol になるため、名前 prefix や文字列 parsing で origin を推測すると、静的検査と codegen 境界に脆い第二命名規約が残る。

## 対象

- `nepl-core/src/hir.rs`
- `nepl-core/src/typecheck/function_check.rs`
- `nepl-core/src/resource/model.rs`
- `nepl-core/src/resource/lower.rs`
- `nepl-core/src/resource/drop_elaboration.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行) では、HIR `passes::insert_drops` を checked Resource IR drop elaboration へ置き換えることを未完了点としている。
- `ResourceDropElaborationPlan` は checked live drop facts を codegen-facing plan として保持するが、関数単位の origin metadata がないと monomorphized function から source-level function へ戻す根拠が文字列規約になる。
- 技術的負債を残さない方針では、function identity は構造化 metadata として保持し、後段が ad hoc な prefix match を持たないようにする必要がある。

## 問題

monomorphize 後の Resource IR function name は specialization symbol であり、source HIR の関数名とは一致しないことがある。`ResourceDropElaborationPlan` が `name` だけを持つ状態では、次の HIR/Wasm drop call 生成で source 関数との対応を復元するために、`foo_Bar` のような mangled name を parsing するか prefix matching する設計へ流れやすい。

## 影響

- generic/specialized function の drop plan を誤った source function へ対応付ける危険がある。
- HIR `passes::insert_drops` 削除後も、codegen boundary に ad hoc な function-name 推測が残る。
- static check の authority を Resource IR に寄せても、drop call 生成側が HIR 再走査や文字列規約へ戻り、Stage 4 完了条件を満たせない。

## 修正方針

- `HirFunction` に `origin_name` を追加し、typecheck で source-level function name を保存する。
- monomorphize は `name` を specialized symbol に変更しても `origin_name` を維持する。
- `ResourceFunction` と `ResourceDropElaborationFunction` へ `origin_name` を伝搬し、drop elaboration plan が function name と source origin を構造化 metadata として保持する。
- Resource IR dump は `name != origin_name` の場合だけ `origin=...` を表示し、通常関数の dump ノイズを増やさない。

## 検証

- `resource_ir_drop_elaboration_plan_preserves_monomorphized_function_origin` を追加し、generic `ignore<Guard>` specialization で HIR `name != origin_name`、Resource IR `origin_name == "ignore"`、drop elaboration plan `origin_name == "ignore"` を確認した。
- 同 regression で non-Copy parameter auto-drop が source binding `_value` を保持することも確認し、function origin metadata と source binding metadata の接続を固定した。
- `cargo test -p nepl-core --test resource_ir resource_ir_drop_elaboration_plan_preserves_monomorphized_function_origin -- --nocapture` で確認する。
- `cargo check -p nepl-core --tests`、`resource_ir_drop_elaboration_plan` 系 focused tests、`resource_ir_compiler_rejects`、`cargo test -p nepl-core --test drop -- --nocapture`、source policy / issue check、`trunk build`、`tests/compiler/drop.n.md` / `shadowing.n.md` / `drop_overwrite.n.md` の wasm runner で確認する。
