---
id: ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A
title: "final initialized check replay still has reprojection bypasses"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/resource_summary_value_cache/initialized_check.rs"
---

# ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A: final initialized check replay still has reprojection bypasses

## 概要

final initialized function check stable cache は 160 関数を replay できたが、RPN code edit で 128 関数の再検査が残り、そのうち 125 件は replay 時の value reprojection bypass になっていた。

今回、function body に現れる型を final check 用 type reprojection boundary へ加え、function-local temporary / storage を含む storage offset place を stable entry へ保存できるようにした。さらに、body hash が同一である final check replay では layout-opaque な generic projection を保存済み projection surface から戻せるようにし、replay は引き続き fail-closed に保った。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs`
- `nepl-core/src/resource/resource_summary_value_cache/initialized_check.rs`

## 根拠

- Zenn の性能追求方針では、純粋性と静的検査を活かして cache により探索空間を削減することが要求されている。
- `tmp/rpn_final_check_cache_code_edit_20260531.json` では、edit delta が `resource_initialized_function_checks=128`、`resource_initialized_function_check_ops=2202`、`initialized_function_check_reprojection_value_type_bypasses=73`、`initialized_function_check_reprojection_value_place_bypasses=52` だった。
- `tmp/rpn_final_check_reprojection_boundary_20260531.json` では、edit delta が `resource_initialized_function_checks=20`、`resource_initialized_function_check_ops=371`、`initialized_function_check_hits=268`、`initialized_function_check_reprojection_value_place_bypasses=0`、`initialized_function_check_reprojection_value_type_bypasses=7` になった。

## 問題

final initialized function check stable entry が、関数 signature には現れない body-local generic state type と、function-local temporary/storage を参照する storage offset place を replay boundary に含めていなかった。そのため、同じ Resource IR body hash の関数であっても final cell / collection slot の型・place を現在 compile の boundary へ戻せず、通常 checker へ戻っていた。

## 影響

Resource IR final check の全関数再実行は減ったが、微小 code edit compile はまだ秒単位であり、0.5 秒未満 / 10ms incremental 目標に届かない。

## 修正方針

ResourceFunctionCheck stable entry の型再投影 boundary と function-local place ordinal boundary を拡張し、diagnostic/auto-drop を保存しない方針を維持したまま replay miss の根本原因を型・place別に潰した。

残った 7 件の type-only bypass は、今回の root cause とは別に `ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9` へ分割する。

## 検証

- `cargo test -p nepl-core stable_initialized_check --lib -- --nocapture`
- `cargo test -p nepl-core initialized_function_check --lib -- --nocapture`
- `cargo check -p nepl-core`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `tmp/rpn_final_check_reprojection_boundary_20260531.json`: edit compile `8254ms -> 6021ms`、`resource_initialized_function_checks 128 -> 20`、`resource_initialized_function_check_ops 2202 -> 371`、`initialized_function_check_reprojection_value_place_bypasses 52 -> 0`、`initialized_function_check_reprojection_value_type_bypasses 73 -> 7`
