---
id: ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9
title: "final initialized check residual type bypasses need stable type provenance"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs
---

# ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9: final initialized check residual type bypasses need stable type provenance

## 概要

final initialized function check replay は body type boundary と function-local place offset の修正後も、7 件の type-only reprojection bypass を残している。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs`

## 根拠

- `tmp/rpn_final_check_reprojection_boundary_20260531.json` の edit delta で、`resource_summary_value_initialized_function_check_reprojection_value_place_bypasses=0` まで減った一方、`resource_summary_value_initialized_function_check_reprojection_value_type_bypasses=7` が残った。
- `ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A` で解消した root cause は body-local type boundary と function-local offset place であり、残件はより狭い type provenance / reject counter の問題として分けて追う。

## 問題

final cell / collection slot state のどの型 surface が再投影に失敗しているかを、現在の counter だけでは特定できない。TypeCtx 全体検索で open generic を拾う緩和は stale hit を招くため採用できず、final check entry が持つべき stable type provenance を失敗箇所単位で特定する必要がある。

## 影響

RPN code edit compile は `resource_initialized_function_checks=20` まで減ったが、まだ `compile_ms=6021` であり 0.5 秒未満 / 10ms incremental 目標に届かない。final check の残件だけでなく raw alias / raw-init recomputation も次の支配項として残っている。

## 修正方針

final cell と collection slot state の type reprojection reject を分ける counter を追加し、必要なら `place.ty`、state payload type、projection result type、collection slot lifecycle type をさらに分解する。その後、失敗している proof surface だけに stable type provenance を追加し、TypeCtx-wide open generic lookup は導入しない。

## 検証

RPN same-session code edit JSON で `initialized_function_check_reprojection_value_type_bypasses` が 7 から 0 に下がり、`unstable_entry`、`dependency`、`diagnostic`、`auto_drop` bypass が増えないことを確認する。
