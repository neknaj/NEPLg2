---
id: ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9
title: "final initialized check residual type bypasses need stable type provenance"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs
---

# ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9: final initialized check residual type bypasses need stable type provenance

## 概要

final initialized function check replay は body type boundary と function-local place offset の修正後も、7 件の type-only reprojection bypass を残していた。

細分 counter を追加した結果、7 件はすべて `projection_result_type` に集中していた。final check entry は Resource IR body hash と stable place surface が同一のときだけ replay されるため、保存済み place type が現在 boundary へ戻せている場合は、その型を final state の proof surface として採用するようにした。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs`

## 根拠

- `tmp/rpn_final_check_reprojection_boundary_20260531.json` の edit delta で、`resource_summary_value_initialized_function_check_reprojection_value_place_bypasses=0` まで減った一方、`resource_summary_value_initialized_function_check_reprojection_value_type_bypasses=7` が残った。
- `tmp/rpn_final_check_residual_type_counter_20260531.json` で、7 件すべてが `resource_summary_value_initialized_function_check_reprojection_value_projection_result_type_bypasses=7` と判明した。
- `tmp/rpn_final_check_residual_type_fix_20260531.json` では、`resource_summary_value_initialized_function_check_reprojection_value_type_bypasses=0`、細分 counter もすべて 0 になった。
- `ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A` で解消した root cause は body-local type boundary と function-local offset place であり、残件はより狭い type provenance / reject counter の問題として分けて追う。

## 問題

final cell / collection slot state の型そのものではなく、place projection の layout 再計算結果と保存済み place type の照合で落ちていた。保存済み place type は stable key と current boundary から再投影済みであり、body hash と stable place surface が一致する final check replay では、この型を Resource IR final state の proof surface として扱える。

## 影響

RPN code edit compile は `resource_initialized_function_checks=13` まで減ったが、まだ `compile_ms=5770` であり 0.5 秒未満 / 10ms incremental 目標に届かない。final check type bypass は解消したため、raw alias / raw-init recomputation が次の支配項である。

## 修正方針

final cell と collection slot state の type reprojection reject を分ける counter を追加し、`place.ty`、projection result type、cell state type、collection slot state type を分解した。projection result type の mismatch は、TypeCtx-wide open generic lookup ではなく、再投影済みの保存済み place type を final state proof surface として採用することで解消した。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core stable_initialized_check --lib -- --nocapture`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `tmp/rpn_final_check_residual_type_fix_20260531.json`: `initialized_function_check_reprojection_value_type_bypasses 7 -> 0`、`resource_initialized_function_checks 20 -> 13`、`resource_initialized_function_check_ops 371 -> 263`
