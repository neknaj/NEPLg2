---
id: ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A
title: "final initialized check replay still has reprojection bypasses"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized.rs"
---

# ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A: final initialized check replay still has reprojection bypasses

## 概要

final initialized function check stable cache は 160 関数を replay できたが、RPN code edit で 128 関数の再検査が残り、そのうち 125 件は replay 時の value reprojection bypass になっている。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized.rs`

## 根拠

- 未記入

## 問題

final initialized function check stable cache は 160 関数を replay できたが、RPN code edit で 128 関数の再検査が残り、そのうち 125 件は replay 時の value reprojection bypass になっている。

## 影響

Resource IR final check の全関数再実行は減ったが、微小 code edit compile はまだ秒単位であり、0.5 秒未満 / 10ms incremental 目標に届かない。

## 修正方針

ResourceFunctionCheck stable entry の型再投影 boundary と function-local place ordinal boundary を拡張し、diagnostic/auto-drop を保存しない方針を維持したまま replay miss の根本原因を型・place別に潰す。

## 検証

RPN same-session code edit JSON で resource_initialized_function_checks が 128 からさらに減り、initialized_function_check_reprojection_value_type_bypasses と place_bypasses が大幅に減ることを確認する。
