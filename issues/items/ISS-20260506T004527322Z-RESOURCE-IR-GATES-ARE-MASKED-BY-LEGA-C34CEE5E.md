---
id: ISS-20260506T004527322Z-RESOURCE-IR-GATES-ARE-MASKED-BY-LEGA-C34CEE5E
title: "Resource IR gates are masked by legacy move_check ordering"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/compiler.rs, nodesrc/test_resource_gate_order.js, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T004527322Z-RESOURCE-IR-GATES-ARE-MASKED-BY-LEGA-C34CEE5E: Resource IR gates are masked by legacy move_check ordering

## 概要

compiler::run_move_check runs passes::move_check::run before Resource IR lowering/cell/borrow/effect/owner gates, so legacy HIR diagnostics can fail-fast and mask the compiler-owned Resource IR authority path.

## 対象

- `nepl-core/src/compiler.rs, nodesrc/test_resource_gate_order.js, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `compiler::run_move_check` は旧 `passes::move_check::run` の diagnostics が空でない場合に即 `CoreError` を返していた。
- そのため Resource IR lowering coverage / cell / borrow / effect / owner gate は、legacy checker が先に失敗した入力では実行されなかった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は Resource IR を move/borrow/lifetime/drop obligation の authority へ移す計画であり、旧 checker は比較用または fallback として扱うべきである。

## 問題

compiler::run_move_check runs passes::move_check::run before Resource IR lowering/cell/borrow/effect/owner gates, so legacy HIR diagnostics can fail-fast and mask the compiler-owned Resource IR authority path.

## 影響

Static check migration remains a two-authority pipeline: Resource IR diagnostics are not guaranteed to be the first enforced model, and self-host planning can accidentally copy legacy HIR move_check behavior instead of the Resource IR design.

## 修正方針

Run Resource IR lowering and all hard gates before legacy passes::move_check::run, keep the legacy checker only as a post-Resource fallback, and add a source policy regression for that ordering.

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core compiler::tests::resource_cell_gate_maps_cell_diagnostics_to_cell_code --lib; node nodesrc/test_resource_gate_order.js; node nodesrc/issues.js check

## 対応内容

- `compiler::run_move_check` の順序を変更し、Resource IR lowering coverage / cell / borrow / effect / owner gate を旧 `passes::move_check::run` より先に実行するようにした。
- 旧 checker は Resource IR gate 通過後の fallback 防壁として残した。
- `nodesrc/test_resource_gate_order.js` を追加し、source policy runner でこの順序を監視するようにした。
- 静的検査計画と soundness review、親 issue `ISS-20260425T000000Z-RV-CORE-009-58589A3F` を更新した。
