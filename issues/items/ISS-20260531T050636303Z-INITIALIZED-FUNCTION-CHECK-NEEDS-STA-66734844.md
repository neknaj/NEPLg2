---
id: ISS-20260531T050636303Z-INITIALIZED-FUNCTION-CHECK-NEEDS-STA-66734844
title: "initialized function check needs stable result cache"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/initialized.rs; nepl-web/src/lib.rs"
---

# ISS-20260531T050636303Z-INITIALIZED-FUNCTION-CHECK-NEEDS-STA-66734844: initialized function check needs stable result cache

## 概要

RPN same-session code edit では raw-init summary replay 後も final initialized function
check が全 reachable function で走っている。unchanged stdlib function の `ResourceFunctionCheck`
相当の stable result を session cache または stdlib prechecked artifact として再利用し、
微小編集時に final check の全関数再走査を避ける。

## 対象

- `nepl-core/src/resource/initialized.rs`
- `nepl-core/src/resource/report.rs`
- `nepl-core/src/resource/resource_summary_value_cache`
- `nepl-web/src/lib.rs`

## 根拠

- `tmp/rpn_stage_breakdown_code_edit_20260531.json` の code edit delta では、`resource_initialized_function_checks=288`、`resource_initialized_function_check_ops=3642` が残っている。
- native release timing では `resource_initialized_function_checks=1875ms` で、`str_trim`、`str_slice_result`、RPN entry functions が上位だった。
- raw-init replay delta は `resource_summary_value_replayed_ops=253`、`resource_summary_value_recomputed_ops=21` まで進んでいるため、final check の全関数再実行が次の独立した支配項である。

## 問題

`check_resource_initialized_moves_inner` は summary fixed-point を作った後、全 reachable
function に対して `ResourceCheckEngine::check_function` を実行する。これは最終診断と
drop/deferred/collection slot state を生成する authority であり削除できないが、
unchanged stdlib function について compile ごとに同じ Resource IR body と同じ summary
inputs で再実行する必要はない。

## 影響

- RPN のような stdlib-heavy workload で code edit compile が秒単位に残る。
- raw-init / i32 scalar summary cache を増やしても、final function check が全関数実行される限り 10ms 微小再compileに近づかない。
- stdlib prechecked artifact の設計では、summary だけでなく final check result の保存境界も必要になる。

## 修正方針

- `ResourceFunctionCheck` のうち stable mirror 化できる surface を定義する。
- key は Resource IR body hash、raw alias / i32 scalar / raw-init / collection-slot summary dependency hash、source capability policy hash、target/profile、typed signature boundary を含める。
- `CellStateEntry`、`CollectionSlotStateEntry`、`auto_drop_points`、`ResourceCheckDeferred` を `TypeId` / `Span` 非依存の stable representation に変換し、現在 compile へ再投影できる場合だけ replay する。
- diagnostics は span を含むため、初期段階では「diagnostic-free function result」だけを cache し、診断がある関数は通常 check に戻す。
- stdlib function は prechecked artifact 化し、user-edited root function とその dependency closure だけを final check する方向へ進める。

## 検証

- focused unit test で diagnostic-free function check result が同じ body / summary inputs で replay されることを確認する。
- function body、summary dependency、source capability policy、signature type の変更で stale replay しないことを確認する。
- RPN same-session code edit JSON で `resource_initialized_function_checks` と `resource_initialized_function_check_ops` が大きく減ることを確認する。
