---
id: ISS-20260505T233519789Z-RESOURCE-EFFECT-COUNTS-EXCEED-RESPON-F1DCE536
title: "Resource effect counts exceed responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_counts.rs, nepl-core/src/resource/effect_counts_raw.rs, nepl-core/src/resource/effect_counts_host.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T233519789Z-RESOURCE-EFFECT-COUNTS-EXCEED-RESPON-F1DCE536: Resource effect counts exceed responsibility split limit

## 概要

After the host effect operation count change, nepl-core/src/resource/effect.rs grew to 297 lines while the resource responsibility policy limit is 160. Effect boundary orchestration, diagnostics, and operation count data are now coupled in one file.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_counts.rs, nepl-core/src/resource/effect_counts_raw.rs, nepl-core/src/resource/effect_counts_host.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- remote main `1818288f` 取り込み後、`node nodesrc/run_source_policy_regressions.js --warn-only` が `effect.rs has 297 lines; responsibility split limit is 160` を報告した。
- `ResourceEffectCounts`、raw memory operation count、host effect operation count、diagnostic/report entry point が `effect.rs` に同居していた。
- effect boundary の入口と count model が同じ file にあると、host operation 追加時の exhaustive match 更新と boundary report 更新の責務が混ざる。

## 問題

After the host effect operation count change, nepl-core/src/resource/effect.rs grew to 297 lines while the resource responsibility policy limit is 160. Effect boundary orchestration, diagnostics, and operation count data are now coupled in one file.

## 影響

Source policy regressions report effect.rs has 297 lines; responsibility split limit is 160. Keeping the counts in effect.rs makes future Resource IR effect changes harder to audit and weakens the module boundary that is supposed to keep static/resource checks reviewable.

## 修正方針

Move ResourceEffectCounts, RawMemoryEffectCounts, ExternalIoEffectCounts, and NondetEffectCounts into focused count modules, keep effect.rs as the boundary report/check entry point, and update the responsibility policy to cover the new modules.

## 検証

- `cargo fmt -p nepl-core`: 実行済み
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_counts_host_effect_operations -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `effect.rs` 超過は解消。次の別件として `initialized_external_io.rs has 156 lines; responsibility split limit is 140` を検出した。

## 対応

- `ResourceEffectCounts` を `effect_counts.rs` に分離した。
- `RawMemoryEffectCounts` を `effect_counts_raw.rs` に分離し、raw memory operation ごとの exhaustive record / total をこの module に閉じた。
- `ExternalIoEffectCounts` / `NondetEffectCounts` を `effect_counts_host.rs` に分離し、host operation ごとの exhaustive record / total をこの module に閉じた。
- `effect.rs` は Resource effect boundary report、diagnostic、check entry point に絞った。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在確認と line limit を追加し、count model の再集約を検出できるようにした。
