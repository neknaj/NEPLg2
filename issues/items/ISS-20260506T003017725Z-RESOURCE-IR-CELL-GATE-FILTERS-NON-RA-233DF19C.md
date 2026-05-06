---
id: ISS-20260506T003017725Z-RESOURCE-IR-CELL-GATE-FILTERS-NON-RA-233DF19C
title: "Resource IR cell gate filters non-raw cell diagnostics"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/compiler.rs, nepl-core/src/resource/report.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T003017725Z-RESOURCE-IR-CELL-GATE-FILTERS-NON-RA-233DF19C: Resource IR cell gate filters non-raw cell diagnostics

## 概要

The compiler Resource IR cell gate converted only raw-memory CellUnavailable operations to diagnostics. Normal Resource IR read, move, drop, call argument, construct input, branch/match and return cell-state violations remained shadow-only after the old move_check passed.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/src/resource/report.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/compiler.rs` は `run_resource_raw_cell_gate` で Resource IR の `CellUnavailable` を処理していたが、`resource_check_operation_is_raw_memory_cell(...)` に合致する operation だけを diagnostic 化していた。
- `ResourceCheckOperation::Read`、`Move`、`Drop`、`CallArgument`、`ConstructInput`、`ReturnValue` などは `check_resource_initialized_moves` が診断として保持していても compiler boundary では捨てられていた。
- `doc/neplg2/static_check_soundness_review_20260430.md` でもこの raw-memory-only gate が Stage 4 の未完了点として記録されていた。

## 問題

The compiler Resource IR cell gate converted only raw-memory CellUnavailable operations to diagnostics. Normal Resource IR read, move, drop, call argument, construct input, branch/match and return cell-state violations remained shadow-only after the old move_check passed.

## 影響

Resource IR cannot become the final authority for move/initialized/drop safety while ordinary cell-state violations are ignored at the compiler boundary. A false negative in the old HIR move_check can still reach later pipeline stages.

## 修正方針

Convert every ResourceCheckDiagnostic::CellUnavailable to a resource.cell.* compiler diagnostic, while keeping the existing compiler-owned raw-memory-boundary allowance limited to migration sources.

## 検証

Run targeted compiler cell-gate unit tests, Resource IR cell tests, cargo fmt/check as needed, node nodesrc/issues.js check, and diff checks.

## 対応結果

2026-05-06 に compiler boundary を `run_resource_cell_gate` へ整理し、`ResourceCheckDiagnostic::CellUnavailable` を raw-memory operation に限定せずすべて `resource.cell.*` compiler diagnostic へ変換するようにした。

- `resource_check_operation_is_raw_memory_cell` を削除し、operation filter による shadow-only 経路をなくした。
- diagnostic message も `raw memory cell ownership violation` から、通常 value cell を含む `resource ir cell state violation` に改めた。
- raw-memory-boundary capability による `stdlib/core/mem.nepl` などの移行中許可は既存通り維持し、user/source boundary 側の検査だけを強めた。
- compiler unit test は raw memory operation と通常 `Read` / `ReturnValue` の両方が `resource.cell.*` code へ写像されることを固定した。

検証:

- `cargo test -p nepl-core compiler::tests::resource_cell_gate_maps_cell_diagnostics_to_cell_code --lib`
