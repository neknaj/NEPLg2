---
id: ISS-20260507T032436743Z-RESOURCE-RAW-CELL-RANGE-MODULE-EXCEE-92BDC72B
title: "Resource range and initialization summary modules exceed responsibility split limits"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/cell_state_raw_range*.rs; nepl-core/src/resource/initialized_alias*.rs; nepl-core/src/resource/initialized_summary*.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T032436743Z-RESOURCE-RAW-CELL-RANGE-MODULE-EXCEE-92BDC72B: Resource range and initialization summary modules exceed responsibility split limits

## 概要

After syncing origin/main 18768838, source policy reported `cell_state_raw_range.rs` over its responsibility split limit. During the root-cause split, the same regression surfaced in adjacent Resource IR proof modules: initialized alias scale/relation helpers and initialized summary return/param/range application had also begun to accumulate distinct proof responsibilities in single files.

## 対象

- `nepl-core/src/resource/cell_state_raw_range*.rs`
- `nepl-core/src/resource/initialized_alias*.rs`
- `nepl-core/src/resource/initialized_summary*.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `origin/main` `18768838` を取り込んだ後、`node nodesrc/run_source_policy_regressions.js --warn-only` が `nodesrc/test_resource_checker_responsibility.js` で検出した。
- 失敗内容は `cell_state_raw_range.rs has 159 lines; responsibility split limit is 140`。
- `cell_state_raw_range.rs` は returned byte range summary の安全性を支える Resource IR の cell-state proof module であり、line limit を上げて隠すのではなく責務分割で解消する必要がある。
- 分割を進めると、`initialized_alias.rs`、`initialized_summary.rs`、`initialized_alias_flow_apply.rs`、`initialized_summary_apply.rs`、`initialized_summary_cells.rs` でも同じ source policy 境界に到達した。これは個別の line 数だけでなく、Resource IR Stage 4 の proof model が監査単位を失い始めている兆候である。
- 関連計画: [`doc/neplg2/static_check_complexity_reduction_plan.md`](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 4 `resource check への移行`。

## 問題

Returned byte range summary、element-size range summary、alias scale/relation fact、return/param initialized summary application が、機能追加ごとに既存 file へ追記されていた。line limit を上げるだけでは memory-safety-critical な Resource IR proof の監査性が下がり、`MemPtr` / raw range / initialized cell の責務分離が再び崩れる。

## 影響

Raw cell range availability is part of initialized-cell proof propagation. Letting this module grow past the responsibility boundary makes Resource IR memory-safety checks harder to audit and can hide future raw range regressions behind broad helper code.

## 修正方針

Split Resource IR proof modules by stable responsibility without raising responsibility limits:

- raw range data model と cell table mutation を分離する。
- range availability / guarded symbolic offset proof を dedicated module に分離する。
- initialized alias relation operation、scale facts、tests を alias table 本体から分離する。
- raw initialization summary の return cell、param cell、return byte range、release model、return application を別 module に分離する。
- `nodesrc/test_resource_checker_responsibility.js` に新しい module 一覧と行数上限を追加し、再集中を回帰として検出する。

## 解決内容

`refactor/resource-raw-range-module-split` で次を実施した。

- `cell_state_raw_range.rs` は `CellTable` の raw range mutation API に集中させ、range 型は `cell_state_raw_range_model.rs`、availability proof は `cell_state_raw_range_cover.rs` に分けた。
- `initialized_alias.rs` から relation predicate と regression tests を分離し、scale fact module も source policy の監視対象にした。
- `initialized_alias_flow_apply.rs` から return summary projection offset substitution を `initialized_alias_flow_projection.rs` へ分離した。
- `initialized_summary.rs` から release requirement model を `initialized_summary_release_model.rs` へ分離した。
- `initialized_summary_apply.rs` から returned initialization summary application を `initialized_summary_apply_return.rs` へ分離した。
- `initialized_summary_cells.rs` は return cell collection に集中させ、param cell collection は `initialized_summary_param_cells.rs`、return byte range collection は `initialized_summary_byte_ranges.rs` へ分離した。
- source policy は全ての新 module の存在、`mod` 宣言、line limit を監視するように更新した。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core i32_scale -- --nocapture`: passed
- `cargo test -p nepl-core element_range_accepts_guarded_scaled_symbolic_offset -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header -- --nocapture`: passed
