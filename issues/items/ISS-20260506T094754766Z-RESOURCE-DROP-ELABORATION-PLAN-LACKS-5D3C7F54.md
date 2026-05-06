---
id: ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54
title: "Resource drop elaboration plan lacks HIR bridge validation"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/drop_elaboration_hir_bridge.rs, nepl-core/src/compiler.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T094754766Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-5D3C7F54: Resource drop elaboration plan lacks HIR bridge validation

## 概要

`PreparedProgram` は `ResourceDropElaborationPlan` を保持するようになったが、各 checked drop point が source HIR の関数・binding・scope span へ戻せるかは検証していなかった。この bridge gate がないまま実 drop call 挿入へ進むと、失敗時に HIR scope discovery や文字列推測へ戻る危険がある。

## 対象

- `nepl-core/src/resource/drop_elaboration_hir_bridge.rs, nepl-core/src/compiler.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行) は、HIR `passes::insert_drops` を checked Resource IR drop elaboration へ置き換えることを未完了点としている。
- checked plan は `origin_name`、`source_name`、typed drop point path を持つが、codegen bridge の直前で source HIR 側の受け皿が存在することを hard gate していなかった。
- 実挿入へ進む前に bridge の前提を enum error で検査しないと、次工程が HIR traversal を暗黙 authority として残す可能性がある。

## 問題

`ResourceDropElaborationPlan` は Resource IR 上では検証済みだが、source HIR module に同じ `origin_name` の関数があるか、drop entry の `source_name` が該当 scope span の parameter / let / match binding として存在するかを確認していなかった。

## 影響

- stale / incomplete な checked plan が Resource IR validation を通過しても、HIR/Wasm bridge で消費不能になる。
- 次工程で bridge 不能な entry を見つけた場合に、HIR scope walker や span fallback を復活させる圧力が残る。
- Stage 4 の「checked Resource IR facts を drop elaboration authority にする」完了条件が曖昧になる。

## 修正方針

- `drop_elaboration_hir_bridge.rs` を追加し、`ResourceDropElaborationPlan` を source HIR module へ戻せるか検証する。
- HIR 側は function parameter、block-local `let`、match arm binding を scope span ごとに収集し、plan の `origin_name` / `source_name` / `span` と照合する。
- `MissingSourceFunction` / `MissingSourceBinding` を enum error とし、compiler では `resource.lower.incomplete` の hard error へ写像する。
- `prepare_module_for_codegen_with_source_map` で HIR `passes::insert_drops` の前に bridge gate を実行する。

## 検証

- `resource_ir_drop_elaboration_hir_bridge_accepts_monomorphized_origin` を追加し、generic specialization 後の plan が source HIR origin / binding に戻せることを確認した。
- `resource_ir_drop_elaboration_hir_bridge_rejects_missing_source_origin` / `resource_ir_drop_elaboration_hir_bridge_rejects_missing_source_binding` で壊れた plan が enum error で拒否されることを確認した。
- `prepare_codegen_exposes_checked_resource_drop_elaboration_plan` と `nodesrc/test_resource_gate_order.js` で compiler pipeline の gate 順序を確認する。
- `cargo check -p nepl-core --tests`、source policy / issue check、`trunk build`、`tests/compiler/drop.n.md` / `shadowing.n.md` / `drop_overwrite.n.md` の wasm runner で確認する。
