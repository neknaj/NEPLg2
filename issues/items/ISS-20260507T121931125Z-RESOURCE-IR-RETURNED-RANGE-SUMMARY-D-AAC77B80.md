---
id: ISS-20260507T121931125Z-RESOURCE-IR-RETURNED-RANGE-SUMMARY-D-AAC77B80
title: "Resource IR returned range summary drops literal count source"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260507T121931125Z-RESOURCE-IR-RETURNED-RANGE-SUMMARY-D-AAC77B80: Resource IR returned range summary drops literal count source

## 概要

Resource IR returned raw byte/element range summaries only record the count as a projection under the returned value. If a callee initializes a returned pointer's pointee with an internal literal count, the summary cannot represent that bound and caller-side RawMemoryLoadCell reports the returned pointee as uninitialized.

## 対象

- `nepl-core/src/resource/initialized_summary*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer` が、callee 内部の `fill_len = 4` を returned initialized range summary の count として caller に渡せず、caller 側 `RawMemoryLoadCell` を `Uninit` と誤診断した。
- `resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell` は同じ周辺 regression だが、こちらは direct ResourceIR fixture が `ResourceOffset::Known` を element index として使っていた。現在の `ResourceOffset::Known` は byte offset なので fixture を byte offset に修正した。

## 問題

Resource IR returned raw byte/element range summaries only record the count as a projection under the returned value. If a callee initializes a returned pointer's pointee with an internal literal count, the summary cannot represent that bound and caller-side RawMemoryLoadCell reports the returned pointee as uninitialized.

## 影響

A valid helper that returns a raw header/pointer after fill_i32 with a literal element count loses initialized-range evidence at the function boundary. This hides ResourceIR full-suite regressions and weakens static-check validation for returned raw storage flows.

## 修正方針

Represent returned/parameter range count sources as typed enums instead of suffix-only fields, and add a first-class i32 constant PlaceRoot so known literal bounds can be carried through summaries without synthetic locals or temporary id sentinels.

## 検証

Focused ResourceIR regressions for raw-cell rekey, returned header literal count summary, and existing guarded/unguarded returned raw header byte ranges must pass.

## 2026-05-07 Agent 1 fixed

根本原因は、`RawCellInitializationReturnByteRange` / `RawCellInitializationParamByteRange` が count を「return value または parameter 配下の projection」としてしか表せない設計だった。callee 内部 literal count のように projection 先を持たないが値は既知の bound は、summary 収集時に落ちていた。

修正:

- returned range count を `RawCellInitializationReturnCount` enum にし、`ReturnValueProjection` と `KnownI32` を明示した。
- parameter / variant parameter range count も `RawCellInitializationParamCount` enum にし、`ParamProjection` と `KnownI32` を明示した。
- `PlaceRoot::I32Constant(i32)` を追加し、known literal bound を synthetic local や temporary id sentinel ではなく first-class scalar place として扱うようにした。
- summary apply / pending variant summary apply は count enum を exhaustive `match` で処理し、known count は `Place::i32_constant` へ materialize する。
- direct ResourceIR fixture は `ResourceOffset::Known` の byte offset 設計に合わせ、i32 の 3 要素目を `Known(8)`、2 要素目を `Known(4)` として固定した。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range -- --nocapture`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `trunk build --release`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-return-count-summary-rebased.json -j 1 --dist web/dist`: total=14, passed=14
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-return-count-summary-rebased.json -j 1 --dist web/dist`: total=110, passed=110
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 217 passed / 11 failed。今回対象の cell / return-summary 2 件は解消し、残りは owner summary 系として `ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F` に継続する。
