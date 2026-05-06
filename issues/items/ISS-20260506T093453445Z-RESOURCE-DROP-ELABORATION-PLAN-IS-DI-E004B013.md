---
id: ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013
title: "Resource drop elaboration plan is discarded before codegen"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/compiler.rs, nepl-core/tests/check_pipeline.rs, nodesrc/test_resource_gate_order.js"
---

# ISS-20260506T093453445Z-RESOURCE-DROP-ELABORATION-PLAN-IS-DI-E004B013: Resource drop elaboration plan is discarded before codegen

## 概要

`ResourceDropElaborationPlan` は `run_resource_static_check` 内で検証されていたが、検証後すぐに捨てられていた。そのため後続の codegen path は checked live drop facts へ構造化アクセスできず、HIR `passes::insert_drops` の scope walker を段階的に置き換える足場がなかった。

## 対象

- `nepl-core/src/compiler.rs`
- `nepl-core/tests/check_pipeline.rs`
- `nodesrc/test_resource_gate_order.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行) は、HIR `passes::insert_drops` を checked Resource IR drop elaboration へ置き換えることを未完了点としている。
- checked plan を compiler pipeline artifact として保持しない限り、次工程は Resource IR の checked live facts ではなく HIR traversal や再計算へ戻るしかない。
- 直前の `origin_name` / source binding / typed drop point path は、plan が後段へ渡って初めて codegen bridge の入力になる。

## 問題

`run_resource_static_check` は `compute_resource_drop_elaboration_plan` を hard gate として実行するだけで、成功した plan を返していなかった。結果として `prepare_module_for_codegen_with_source_map` は checked Resource IR drop facts を保持せず、`passes::insert_drops` の置換作業が compiler pipeline 上の正式な入力を持てない。

## 影響

- 後続の drop call 生成が `ResourceFunctionCheck::auto_drop_points` ではなく candidate plan や HIR scope walker を再 authority にしてしまう。
- function origin / source binding / typed drop point path を追加しても、pipeline で保持されなければ codegen bridge で使えない。
- self-host 実装時に「検証だけして捨てる plan」をコピーし、drop elaboration の責務分割が未完のまま残る。

## 修正方針

- `run_resource_static_check` の戻り値を `ResourceDropElaborationPlan` に変更し、drop elaboration plan gate 成功時に checked plan を返す。
- `PreparedProgram` に `resource_drop_elaboration_plan` を追加し、codegen prepared artifact が Resource IR checked live drop facts を保持する。
- `nodesrc/test_resource_gate_order.js` で `prepare_module_for_codegen_with_source_map` が checked plan を捨てずに `PreparedProgram` へ渡すことを監視する。

## 検証

- `prepare_codegen_exposes_checked_resource_drop_elaboration_plan` を追加し、generic `ignore<Guard>` の monomorphized function について `PreparedProgram.resource_drop_elaboration_plan` が source origin と `_value` の checked drop fact を保持することを確認した。
- `cargo test -p nepl-core --test check_pipeline prepare_codegen_exposes_checked_resource_drop_elaboration_plan -- --nocapture` で確認する。
- `cargo check -p nepl-core --tests`、source policy / issue check、`trunk build`、`tests/compiler/drop.n.md` / `shadowing.n.md` / `drop_overwrite.n.md` の wasm runner で確認する。
