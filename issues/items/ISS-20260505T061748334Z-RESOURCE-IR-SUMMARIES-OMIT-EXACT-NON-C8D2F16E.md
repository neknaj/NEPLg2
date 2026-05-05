---
id: ISS-20260505T061748334Z-RESOURCE-IR-SUMMARIES-OMIT-EXACT-NON-C8D2F16E
title: "Resource IR summaries omit exact non-Copy raw load moves"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_summary*.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T061748334Z-RESOURCE-IR-SUMMARIES-OMIT-EXACT-NON-C8D2F16E: Resource IR summaries omit exact non-Copy raw load moves

## 概要

Function summaries record raw-cell initialization and destructive raw-memory checks, but they do not propagate that a callee moved a non-Copy value out of an exact raw cell through load<T>. A caller that stores a non-Copy value, calls such a helper, and then deallocates the storage still sees the raw cell as initialized.

## 対象

- `nepl-core/src/resource/initialized_summary*.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/resource/initialized_summary_destruction_address.rs` は `RawMemoryOp::Store` / `Dealloc` / `Realloc` / `Fill` / `Bulk*` を param destruction summary として収集していたが、non-Copy `RawMemoryOp::Load` を caller 側の raw cell move として収集していなかった。
- `nepl-core/src/resource/initialized_summary_apply.rs` は callee summary の param destruction を call site で確認するだけで、callee が exact raw cell から non-Copy 値を move-load した事実を caller の `CellTable` に反映していなかった。
- そのため `store<T> p value; take_exact p; dealloc_raw p size` のような正当な exact cell move helper が、caller 側では `p.deref` を `Initialized(T)` のまま残し、dealloc 時に `resource.cell.initialized_conflict` になっていた。

## 問題

Function summaries record raw-cell initialization and destructive raw-memory checks, but they do not propagate that a callee moved a non-Copy value out of an exact raw cell through load<T>. A caller that stores a non-Copy value, calls such a helper, and then deallocates the storage still sees the raw cell as initialized.

## 影響

Valid owner-consuming helper boundaries are rejected with resource.cell.initialized_conflict, while uninitialized exact raw-cell loads through helper summaries are not diagnosed at the call site. This weakens the Resource IR authority needed for safe collection cleanup.

## 修正方針

Add an explicit param raw-cell move summary for non-Copy RawMemory::Load on exact parameter-derived addresses, apply it at direct and indirect call sites with availability checking, and keep unknown-offset/range loads conservative for the larger range cleanup issue.

## 検証

Add Resource IR regressions for exact non-Copy raw load summary propagation and uninitialized call-site rejection; run the focused resource_ir tests.

## 対応結果

`RawCellInitializationFunctionSummary` に `param_moves` を追加し、callee が parameter-derived exact raw address から non-Copy 値を `RawMemoryOp::Load` した事実を caller へ伝播するようにした。

- `RawCellMoveParamAddress` は parameter index、address suffix、address type、cell type、diagnostic operation を持つ。
- summary collection は `RawMemoryOp::Load` かつ output type が non-Copy の場合だけ param move を収集する。
- callee 内で対象 exact raw cell が既に initialized になっている `load<T>` は caller の事前条件ではないため、summary 収集時の `CellTable` で availability を確認し、callee-initialized load は `param_moves` から除外する。
- unknown offset を含む address は range cleanup の別設計が必要なため、今回の exact move summary では収集しない。これは `ISS-20260505T045316820Z-RESOURCE-IR-VEC-RANGE-CLEANUP-9A95DB6F` の残件として保守的に維持する。
- direct call / indirect call の summary 伝播にも param move を含めた。
- call site では対象 raw cell の availability を `RawMemoryLoadCell` として確認し、成功した場合だけ caller 側の raw cell を moved に更新する。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_summary -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_destructive_raw_storage_ops_over_live_cell -- --nocapture`: passed
- `cargo fmt --check`: passed
