---
id: ISS-20260513T063212027Z-SOURCE-POLICY-RESOURCE-GATE-ORDER-ST-ED44ABFD
title: "source policy resource gate order still expects raw-boundary source_map in cell and owner gates"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nodesrc/test_resource_gate_order.js,nepl-core/src/compiler.rs"
---

# ISS-20260513T063212027Z-SOURCE-POLICY-RESOURCE-GATE-ORDER-ST-ED44ABFD: source policy resource gate order still expects raw-boundary source_map in cell and owner gates

## 概要

Resource cell/owner gate から raw-memory-boundary 判定を外し、boundary capability の判断を effect boundary gate に限定した後も、test_resource_gate_order.js が run_resource_cell_gate と run_resource_owner_obligation_gate に source_map 引数を要求している。

## 対象

- `nodesrc/test_resource_gate_order.js,nepl-core/src/compiler.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` で `nodesrc/test_resource_gate_order.js` が `run_resource_cell_gate(&initialized_moves, diagnostics, source_map)` を要求して失敗していた。
- 現在の `nepl-core/src/compiler.rs` は cell/owner gate を SourceMap 非依存にし、raw-memory-boundary capability による抑制を `run_resource_effect_boundary_gate` へ限定している。

## 問題

Resource cell/owner gate から raw-memory-boundary 判定を外し、boundary capability の判断を effect boundary gate に限定した後も、test_resource_gate_order.js が run_resource_cell_gate と run_resource_owner_obligation_gate に source_map 引数を要求している。

## 影響

静的検査の責務分離に反する古い source-policy が warn-only で残り、cell/owner gate に raw-boundary 抑制を戻すような誤った修正を誘導し得る。

## 修正方針

Resource IR static check の gate 順序は維持しつつ、cell/owner gate は diagnostics のみを受け取り SourceMap を使わないこと、effect boundary gate だけが source_map を受け取ることを policy に反映する。

## 検証

- `node nodesrc/test_resource_gate_order.js`

## 2026-05-13 修正

`nodesrc/test_resource_gate_order.js` の gate 順序 policy を現在の静的検査責務分離へ合わせた。cell gate と owner obligation gate は diagnostics のみを受け取ることを要求し、古い `source_map` 引数付き呼び出しを明示的に禁止した。effect boundary gate は引き続き `source_map` を受け取り、raw-memory-boundary capability の判断が effect 境界に集中していることを確認する。
