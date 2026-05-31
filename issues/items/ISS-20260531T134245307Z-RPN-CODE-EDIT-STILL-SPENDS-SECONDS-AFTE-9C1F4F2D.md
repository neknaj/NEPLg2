---
id: ISS-20260531T134245307Z-RPN-CODE-EDIT-STILL-SPENDS-SECONDS-AFTE-9C1F4F2D
title: "RPN code edit still spends seconds after raw-init replay"
area: core
status: investigating
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource; nepl-web/src/lib.rs; nodesrc/run_test.js"
---

# ISS-20260531T134245307Z-RPN-CODE-EDIT-STILL-SPENDS-SECONDS-AFTE-9C1F4F2D: RPN code edit still spends seconds after raw-init replay

## 概要

Complete raw-init leaf replay の false miss は解消したが、RPN same-session code edit はまだ数秒かかる。`recomputed_ops=21` と Resource IR summary cache 外の固定費を分解し、0.5 秒未満の compile と 10ms 未満の微小再compileへ近づける必要がある。

## 対象

- `nepl-core/src/resource`
- `nepl-web/src/lib.rs`
- `nodesrc/run_test.js`

## 根拠

- 2026-05-31 の `tmp/rpn_return_type_canonicalization_code_edit_20260531.json` では、base `compile_ms=8861`、edit `compile_ms=6703`。
- 同 edit delta では `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=253`、`raw_init_param_facts_bypasses=0`、`raw_init_param_facts_reprojection_value_bypasses=0`、`param_cell_result_type=0` まで改善した。
- それでも edit delta に `resource_summary_value_recomputed_ops=21` が残り、compile time は秒単位である。
- raw-init complete leaf replay だけでは、stdlib-heavy workload の typecheck / monomorphize / Resource IR summary build / codegen の残り固定費を消せない。

## 問題

現在の timing は raw-init replay が効いたことは示すが、replay 後にどの stage / function / summary kind が秒単位の時間を消費しているかを十分に分解できていない。次の性能改善では、remaining 21 ops と Resource IR summary cache 外の固定費を測定し、根本原因ごとに issue を分ける必要がある。

## 修正方針

- RPN same-session code edit の stage timing と Resource IR per-function timing を再取得する。
- `resource_summary_value_recomputed_ops=21` の function / summary kind / dependency reason を観測できる counter または debug-only timing を追加する。
- raw-init 以外の summary kind、typecheck / monomorphize / codegen fragment cache、stdlib prechecked artifact のどれが次の支配項かを切り分ける。
- timing 追加は通常実行の重さやコメント増加を妨げないよう、明示的な測定モードまたは軽い集約 counter に限定する。

## 2026-05-31 調査更新

Native release CLI の `NEPL_COMPILE_STAGE_TIMING=1` / `NEPL_RESOURCE_PER_FUNCTION_TIMING=1`
で `examples/rpn.nepl` を測定した結果、`resource_static_check=6950ms` の主成分は
`resource_initialized_moves=6050ms` だった。内訳は
`resource_initialized_raw_init_summaries=2502ms`、
`resource_initialized_i32_scalar_summaries=1558ms`、
`resource_initialized_function_checks=1875ms` である。

per-function timing では、`raw_init_summary` は `apply_op` / `dealloc_raw` /
`byte_builder_*`、`i32_scalar_summary` は `sb_append_result` / `byte_builder_reserve`、
final initialized check は `str_trim` / `str_slice_result` / RPN entry 関数が上位だった。
したがって、raw-init complete leaf replay の false miss 解消後の次段階は、少なくとも
次の 3 系統に分けて扱う必要がある。

- i32 scalar summary の stable mirror / replay。
- final initialized function check の function-level stable result cache または stdlib prechecked artifact。
- raw-init summary cache maintenance / remaining recomputation の詳細分解。

この checkpoint では、`CompilerSession.loader_cache_stats_json()` に initialized-state
summary stage の再計算数、summary count、final function check の関数数と op 数を追加する。
これにより Web / Node の same-session code edit JSON でも、raw-init replay 後に
全関数の fixed-point / final check が残っているかを継続観測できる。

`tmp/rpn_stage_breakdown_code_edit_20260531.json` では、base `compile_ms=8919`、
unused local 追加 edit `compile_ms=6771` だった。edit delta は次の通り。

- `resource_raw_alias_summary_recomputations=288`, `resource_raw_alias_summary_count=54`
- `resource_i32_scalar_summary_recomputations=209`, `resource_i32_scalar_summary_count=87`
- `resource_raw_init_summary_recomputations=81`, `resource_raw_init_summary_count=78`
- `resource_collection_slot_summary_recomputations=0`, `resource_collection_slot_summary_count=0`
- `resource_initialized_function_checks=288`, `resource_initialized_function_check_ops=3642`
- `resource_summary_value_replayed_ops=253`, `resource_summary_value_recomputed_ops=21`

この結果に基づき、i32 scalar summary replay を
`ISS-20260531T050630951Z-I32-SCALAR-SUMMARY-NEEDS-STABLE-MIRR-E70E2D93`、
final initialized function check replay を
`ISS-20260531T050636303Z-INITIALIZED-FUNCTION-CHECK-NEEDS-STA-66734844`
へ分割した。

## 検証

- RPN same-session code edit の compiled-output miss 測定で、支配 stage と function / summary kind を説明できる JSON を残す。
- 修正後の測定で `compile_ms`、`recomputed_ops`、または特定 stage timing が改善していることを確認する。
