---
id: ISS-20260505T083557206Z-DYNAMIC-RAW-LOAD-CLEARS-EXACT-INITIA-C9619723
title: "dynamic raw load clears exact initialized cells"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/cell_state.rs,nepl-core/src/resource/initialized_raw_memory.rs,nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T083557206Z-DYNAMIC-RAW-LOAD-CLEARS-EXACT-INITIA-C9619723: dynamic raw load clears exact initialized cells

## 概要

A non-Copy raw load through a dynamic storage offset can remove all exact raw cell facts that may alias the dynamic address. One unknown-offset load is not proof that every exact cell in the range was moved, so a later storage dealloc can be accepted even though an initialized non-Copy cell may still be live.

## 対象

- `nepl-core/src/resource/cell_state.rs,nepl-core/src/resource/initialized_raw_memory.rs,nepl-core/tests/resource_ir.rs`

## 根拠

- `CellTable::mark_raw_cell_moved` が dynamic storage offset の address でも `raw_cell_belongs_to_address_cell` に一致する exact raw cell facts を `retain` で削除し、その後 dynamic cell を `Moved` にしていた。
- `place_suffix_after_address_prefix` は liveness / overlap 判定のために `ResourceOffset::Dynamic` を exact offset と may-alias させるが、同じ may-alias 関数が initialized fact の伝播にも使われていた。そのため exact initialized cell が 1 個でもあれば dynamic raw load を initialized と誤認し、1 回の unknown-offset load が range 全体の cleanup 証明のように扱われ得た。
- 修正前の focused regression では、2 個の exact non-Copy cell を store した後に 1 回だけ dynamic offset から load し、そのまま base storage を dealloc するケースを拒否できなかった。

## 問題

A non-Copy raw load through a dynamic storage offset can remove all exact raw cell facts that may alias the dynamic address. One unknown-offset load is not proof that every exact cell in the range was moved, so a later storage dealloc can be accepted even though an initialized non-Copy cell may still be live.

## 影響

This is a memory-safety false negative in Resource IR. It can hide leaks or invalid shallow dealloc in Vec-style storage cleanup and would make future range cleanup unsound by confusing one dynamic element access with whole-range cleanup.

## 修正方針

Separate may-alias overlap checks from must-alias initialized fact flow. Exact initialized cells must not prove that a dynamic address is initialized unless the exact same dynamic cell fact exists. When a dynamic-offset non-Copy load is accepted, mark overlapping exact cells and the dynamic cell as MaybeMoved instead of deleting exact facts, while keeping exact loads precise.

## 検証

Add a Resource IR regression where two exact non-Copy cells exist, one unknown-offset load occurs, and dealloc must still report a live/maybe-moved cell. Run focused exact/dynamic resource_ir tests.

## 対応結果

`CellTable` の address relation を、liveness / overlap 用の may-alias と initialized fact 伝播用の must-alias に分離した。dynamic offset は dealloc や destructive store では exact offset と alias し得るものとして保守的に扱う一方、exact initialized fact から dynamic raw load の initialized availability は証明しない。

dynamic-offset non-Copy load が availability を満たして実行される場合も、overlap する exact initialized cells は削除せず `MaybeMoved` として残す。dynamic cell 自体も `MaybeMoved` として記録し、後続 dealloc / reload は live obligation として拒否できるようにした。exact offset load は従来通り対象 cell だけを precise に `Moved` へ落とす。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_unknown_offset_load_does_not_clear_all_exact_cells -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_summary_rejects_unproven_unknown_offset_non_copy_raw_load -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_zero_alias_reports_moved_cell -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_unknown_arithmetic_helper_offset_conservative -- --nocapture`
- `cargo check -p nepl-core --tests`
