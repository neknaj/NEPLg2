---
id: ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903
title: "Resource IR returned aggregate fields do not carry initialized raw range summaries"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/src/resource/cell_state_raw_range_value.rs, nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903: Resource IR returned aggregate fields do not carry initialized raw range summaries

## 概要

Raw range summary collection can represent a raw header return, but aggregate field projection across a returned struct is still incomplete for address/count pairs. The fd_read bounded range fix exposed this because the previous unbounded payload fact had hidden the missing projection.

## 対象

- `nepl-core/src/resource/initialized_summary_byte_ranges.rs, nepl-core/src/resource/initialized_alias_flow_value_projection.rs, nepl-core/tests/kp.rs`

## 根拠

When local scanner returned a struct containing both a raw header pointer and its owned buffer pointer, caller-side guarded payload loads could not use the callee initialized range summary.

## 問題

Raw range summary collection can represent a raw header return, but aggregate field projection across a returned struct is still incomplete for address/count pairs. The fd_read bounded range fix exposed this because the previous unbounded payload fact had hidden the missing projection.

## 影響

Full scanner/self-host input structures that return metadata structs still need fixture reshaping or additional summary model support. This is a remaining parent issue for returned header / fd_read / capacity integration.

## 修正方針

Extend returned aggregate value projection summaries so address suffix and count suffix can be projected through struct fields without broadening raw memory initialization.

## 検証

Add a focused returned-aggregate scanner/header regression that passes without unknown-offset payload initialization.

## 2026-05-07 修正結果

値コピー時に initialized raw range の address / count projection の片側だけを複写していたことが原因だった。`make_scanner` の戻り値では callee summary が `return.buf` / `return.len` の dependent range を持てても、caller 側で call output を local aggregate へ束縛する段階で range address が call temporary に残り、`field::get sc "buf"` 後の guarded load が `RawMemoryLoadCell Uninit` になっていた。

`CellTable::copy_initialized_raw_byte_ranges_through_value` を追加し、値コピー時に address と count の両方を projection ごと複写するようにした。`DeclareLocal` / `Read` / `Assign` / `Move` / branch / match / raw memory `Load` / raw memory `Store` / aggregate `Construct` に接続し、構造体 field、raw memory cell、関数返却 temporary をまたいでも dependent initialized range が失われない。assignment / raw memory store では overwritten target 配下の古い range fact を消し、stale fact で unfilled buffer が通らないようにした。

この修正は raw memory load の判定を緩めていない。caller 側の `load_u8 add data i` は `0 <= i` と `i < len` が Resource IR relation fact から証明された場合だけ通り、guard なしの symbolic load は引き続き `resource.cell.uninit` になる。

確認:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_aggregate -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_aggregate_assignment_clears_stale_byte_range -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
- `cargo test -p nepl-core element_range_accepts_guarded_scaled_symbolic_offset -- --nocapture`: passed
- `node nodesrc/issues.js check`: passed

追加で `node nodesrc/test_resource_checker_responsibility.js` を確認したところ、今回の raw range value projection 分割後、別件として `initialized_external_io_effect.rs` の上限超過が前面化した。これは `ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730` として分離した。
