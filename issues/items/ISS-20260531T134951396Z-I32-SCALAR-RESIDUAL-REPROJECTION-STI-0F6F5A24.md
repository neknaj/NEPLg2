---
id: ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24
title: "i32 scalar residual reprojection still recomputes RPN edit"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/initialized_scalar_flow.rs; nepl-core/src/resource/resource_summary_value_cache/i32_scalar.rs"
---

# ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24: i32 scalar residual reprojection still recomputes RPN edit

## 概要

RPN same-session string literal edit では、広い Resource summary preseed replay 後も
i32 scalar summary の残差再計算が残っている。kind 別 recomputed-op counter により、
残る `resource_summary_value_recomputed_ops=+16` は i32 scalar return facts に属し、
同じ edit delta の i32 scalar reprojection value bypass `+16` と一致することを確認した。

## 対象

- `nepl-core/src/resource/initialized_scalar_flow.rs`
- `nepl-core/src/resource/resource_summary_value_cache/i32_scalar.rs`

## 根拠

- `tmp/rpn_recomputed_ops_kind_counters_20260531.json` では、base `compile_ms=9237`、edit `compile_ms=3310`。
- base から edit への差分は `resource_summary_value_recomputed_ops=+16`。
- 同じ差分は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16` だけに出ている。
- `resource_summary_value_raw_alias_return_entry_recomputed_ops`、`resource_summary_value_raw_init_param_facts_recomputed_ops`、`resource_summary_value_drop_traversal_forall_recomputed_ops` の差分は `0`。
- `resource_summary_value_i32_scalar_return_facts_bypasses=+16` と `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16` も同時に増えている。

## 問題

現在の aggregate `resource_summary_value_recomputed_ops` だけでは、残差がどの summary kind に
属するかを判断できない。今回追加した kind 別 counter により、残差は i32 scalar の
再投影失敗に限定されることが分かったが、entry 単位 / function 単位 / reason 単位の
詳細はまだ不足している。

このまま changed-function-only proof replay の実装へ進むと、実際には i32 stable mirror の
再投影境界が足りないだけの問題を、scope 設計の問題として誤って扱うおそれがある。

## 影響

- RPN code edit の `resource_static_check` はまだ秒単位であり、changed-function-only proof replay と並行して残差の内訳を正確に追う必要がある。
- i32 scalar summary は raw-init / collection slot / private cache proof の下流入力にもなるため、再投影できない stable mirror surface を放置すると、後続の cache 設計で同じ失敗が再発する。
- 残差は全体の支配項ではないが、aggregate counter に混ざると次の性能 issue 分割を誤らせる。

## 修正方針

- i32 scalar replay miss に、entry 単位 / function 単位 / reason 単位 counter を追加する。
- `ReturnProjection`、`ParameterProjection`、`ScalarType` などの再投影失敗理由を JSON で観測できるようにする。
- 必要なら debug-only 測定モードで、bypass した function name と fact count を出す。
- 原因が scalar type canonicalization など単一 surface に収束したら、dependency closure、source policy、type boundary key を弱めずに stable mirror boundary を修正する。

## 検証

- RPN same-session edit JSON で、i32 scalar recomputed op delta と i32 scalar reprojection bypass delta が `0` になる。
- それができない場合は、関数名 / reason / fact kind を持つさらに狭い follow-up issue に分離されている。
