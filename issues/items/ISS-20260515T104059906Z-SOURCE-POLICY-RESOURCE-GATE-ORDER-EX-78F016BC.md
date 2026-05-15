---
id: ISS-20260515T104059906Z-SOURCE-POLICY-RESOURCE-GATE-ORDER-EX-78F016BC
title: "source policy resource gate order expects untyped effect boundary"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nodesrc/test_resource_gate_order.js,nepl-core/src/compiler.rs"
---

# ISS-20260515T104059906Z-SOURCE-POLICY-RESOURCE-GATE-ORDER-EX-78F016BC: source policy resource gate order expects untyped effect boundary

## 概要

nodesrc/test_resource_gate_order.js still requires run_resource_static_check to call crate::resource::check_resource_effect_boundaries(&resource), but compiler.rs now correctly calls check_resource_effect_boundaries_typed(&resource, types) so Resource IR effect checking can use typed function/effect information.

## 対象

- `nodesrc/test_resource_gate_order.js,nepl-core/src/compiler.rs`

## 根拠

- `node nodesrc/test_resource_gate_order.js` が `run_resource_static_check must call crate::resource::check_resource_effect_boundaries(&resource)` で失敗した。
- 現在の `nepl-core/src/compiler.rs` は `check_resource_effect_boundaries_typed(&resource, types)` を呼び、Resource IR effect boundary を `TypeCtx` 付きで検査している。
- untyped helper を policy に要求し続けると、typed indirect call / function effect summary を使う現行設計と source policy が逆方向になる。

## 問題

nodesrc/test_resource_gate_order.js still requires run_resource_static_check to call crate::resource::check_resource_effect_boundaries(&resource), but compiler.rs now correctly calls check_resource_effect_boundaries_typed(&resource, types) so Resource IR effect checking can use typed function/effect information.

## 影響

The source policy runner reports a stale failure and can push future fixes toward dropping typed effect-boundary checking. That would weaken static-check coverage for function/indirect-call effects and make the policy disagree with the current Resource IR design.

## 修正方針

Update the policy to require check_resource_effect_boundaries_typed(&resource, types), forbid falling back to the untyped effect boundary helper inside run_resource_static_check, and keep the existing gate-order/source_map assertions.

## 検証

node nodesrc/test_resource_gate_order.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check

## 2026-05-15 修正

`nodesrc/test_resource_gate_order.js` を現在の Resource IR effect boundary 設計へ合わせた。

- `run_resource_static_check` が `check_resource_effect_boundaries_typed(&resource, types)` を呼ぶことを必須化した。
- `check_resource_effect_boundaries(&resource)` へ戻す fallback を禁止した。
- cell / owner gate が `source_map` を受け取らず、raw-memory-boundary capability 判定を effect boundary gate に閉じる既存 policy は維持した。
