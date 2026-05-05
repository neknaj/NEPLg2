---
id: ISS-20260505T232450417Z-RESOURCE-HOST-EFFECT-COUNTS-COLLAPSE-C723A6B3
title: "Resource host effect counts collapse typed operations"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T232450417Z-RESOURCE-HOST-EFFECT-COUNTS-COLLAPSE-C723A6B3: Resource host effect counts collapse typed operations

## 概要

ExternalIoOp and NondetOp are typed, but ResourceEffectCounts still stores only total external_io_ops and nondet_ops. Resource effect reports cannot distinguish fd_read, fd_write, path_open, random_get, or clock reads.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceEffectCounts` は raw memory については operation-level count を保持していたが、ExternalIo / Nondet は合計値だけを保持していた。
- `EffectOp::{ExternalIo,Nondet}` が typed enum を保持するようになった後も、`ResourceEffectBoundaryEngine::check_effect` は enum operation を記録せず `usize` を増やすだけだった。
- この状態では `fd_read` と `fd_write`、`random_get` と `clock_time_get` を report から区別できず、Stage 5 effect boundary の監査 evidence が不足する。

## 問題

ExternalIoOp and NondetOp are typed, but ResourceEffectCounts still stores only total external_io_ops and nondet_ops. Resource effect reports cannot distinguish fd_read, fd_write, path_open, random_get, or clock reads.

## 影響

Stage 5 effect boundary reports lose operation-level evidence for host effects. New host effect operations can be added without report-side match coverage, weakening auditability around effect safety and Resource IR enforcement.

## 修正方針

Introduce ExternalIoEffectCounts and NondetEffectCounts with exhaustive record methods, update effect boundary checking to record typed operations, and add focused Resource IR regression for host operation counts.

## 対応

- `ExternalIoEffectCounts` と `NondetEffectCounts` を追加し、host operation ごとの count を保持するようにした。
- `record` は `ExternalIoOp` / `NondetOp` の exhaustive match で更新し、新しい operation 追加時に report 側の更新漏れを検出できる形にした。
- Resource effect boundary check は `EffectOp::{ExternalIo,Nondet}` の typed operation を個別 count に記録するようにした。
- focused Resource IR regression で `fd_read` / `fd_write` / `path_open` / `random_get` / `clock_time_get` の内訳と total を固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_counts_host_effect_operations -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_ -- --nocapture`: 21 passed
- `node nodesrc/issues.js check`: commit 前に実行
