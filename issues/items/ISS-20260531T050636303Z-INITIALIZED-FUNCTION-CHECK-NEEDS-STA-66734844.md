---
id: ISS-20260531T050636303Z-INITIALIZED-FUNCTION-CHECK-NEEDS-STA-66734844
title: "initialized function check needs stable result cache"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-06-01
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

## 2026-05-31 checkpoint

diagnostic-free かつ `auto_drop_points` を持たない `ResourceFunctionCheck` だけを stable entry として保存する MVP を実装した。保存対象は `final_cells`、`final_collection_slots`、`ResourceCheckDeferred` に限定し、diagnostic span と drop elaboration 用 span は cache しない。

key には Resource IR body hash、source capability policy hash、typed signature/type boundary、dependency closure hash を含める。`ResourceId` / `StorageId` は stable value へ直接保存せず、関数本文内の出現順 ordinal として保存し、replay 時に現在の同じ body から実 id へ戻す。

focused regression:

- 同一 diagnostic-free function は二回目 compile で `ResourceCheckEngine` を再実行しない。
- function body が変わると replay miss になり、通常 checker に戻る。
- `auto_drop_points` を持つ entry は no-store になる。

RPN same-session code edit 測定 `tmp/rpn_final_check_cache_code_edit_20260531.json` では、edit delta が `resource_initialized_function_checks=128`、`resource_initialized_function_check_ops=2202`、`resource_summary_value_initialized_function_check_hits=160`、`resource_summary_value_replayed_ops=2122` になった。直前の i32 scalar checkpoint の `resource_initialized_function_checks=288` からは減ったが、まだ `initialized_function_check_reprojection_value_type_bypasses=73`、`initialized_function_check_reprojection_value_place_bypasses=52` が残る。

この issue は partial implementation として継続する。残った replay bypass は `ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A` に分離して追跡する。

## 2026-06-01 changed-function pass plan checkpoint

same-session edit で全関数の final initialized check replay probe を行う固定費を削るため、`InitializedFunctionCheckPassPlan` を追加した。前回 compile で diagnostic-free / auto-drop-free pass として保存できた関数について、現在 compile の関数本文 hash、type boundary、source capability policy、generic boundary、関数 identity を比較し、変化した関数から reverse dependents を辿って affected set を作る。

affected ではない関数は、dependency closure が変わっていないものとして、dependency closure hash を再構築せずに checked pass を戻す。snapshot に保存する値は `ResourceCheckDeferred` と安定 fingerprint だけであり、`TypeId`、`Span`、`SourceMap`、final cell state は保存しない。fingerprint を作れない場合や関数順序・namespace が変わる場合は conservative-all に戻る。

focused regression では、同一 diagnostic-free function の二回目 compile が通常 replay probe ではなく plan skip として観測されること、本文変更では plan skip せず通常 checker に戻ること、callee body edit では reverse dependent caller も affected になることを確認した。さらに namespace 不一致、関数順序変更、source capability policy 変更では前回 pass を再利用しないことを固定した。

この checkpoint は final initialized check の pass-only replay に限定する。raw alias / i32 scalar / raw-init summary fixed-point では、affected 関数の計算に必要な callee summary を materialize する必要があるため、単純な全関数 preseed loop 削減はまだ行わない。
