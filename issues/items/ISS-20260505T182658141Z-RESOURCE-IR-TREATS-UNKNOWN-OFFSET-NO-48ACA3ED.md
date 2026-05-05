---
id: ISS-20260505T182658141Z-RESOURCE-IR-TREATS-UNKNOWN-OFFSET-NO-48ACA3ED
title: "Resource IR treats unknown-offset non-Copy raw moves as exact cleanup"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T182658141Z-RESOURCE-IR-TREATS-UNKNOWN-OFFSET-NO-48ACA3ED: Resource IR treats unknown-offset non-Copy raw moves as exact cleanup

## 概要

Resource IR CellState treats a non-Copy raw load through an unknown storage offset as if the exact cell was fully moved and removed. When dynamic raw storage stores collapse into an unknown-offset cell, a later dynamic load can erase the only live non-Copy obligation and allow storage-only dealloc even though the compiler has not proven full range cleanup.

## 対象

- `nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `CellTable::mark_raw_cell_moved` は raw load の address に unknown storage offset が含まれる場合でも、重なる raw cell entry を削除してから moved cell だけを記録していた。
- unknown-offset dynamic store は複数要素の range fact ではなく単一の曖昧な cell fact へ畳まれるため、その後の dynamic load で entry を削除すると、`dealloc_raw` が live non-Copy obligation を見失う。
- `availability_state` も non-initialized raw cell state を exact prefix だけで見ており、unknown-offset `MaybeMoved` が exact/dynamic alias 側の後続 load に流れない構造だった。

## 問題

Resource IR CellState treats a non-Copy raw load through an unknown storage offset as if the exact cell was fully moved and removed. When dynamic raw storage stores collapse into an unknown-offset cell, a later dynamic load can erase the only live non-Copy obligation and allow storage-only dealloc even though the compiler has not proven full range cleanup.

## 影響

This is a memory-safety false negative around Vec-style dynamic element cleanup: Resource IR can accept dealloc after an unproven dynamic non-Copy move, hiding use-after-move/leak/double-drop bugs behind an imprecise range model.

## 修正方針

Represent unknown-offset non-Copy raw moves conservatively. Exact raw moves may mark only the exact cell moved, but any move involving unknown storage offset must leave overlapping raw cell facts as MaybeMoved and make availability/dealloc checks treat those MaybeMoved facts as overlapping exact/dynamic aliases. Do not weaken RawMemoryLoadCell or special-case stdlib names.

## 検証

Add Resource IR regression showing dynamic non-Copy store/load/dealloc reports RawMemoryDeallocCell MaybeMoved; keep exact disjoint raw offset and existing unknown-offset conservative tests passing.

## 修正結果

- unknown-offset が絡まない exact raw move は従来どおり exact cell を `Moved` にする。
- move 元または既存 raw cell fact に `StorageOffset(None)` が含まれる場合は、重なる raw cell fact を削除せず `MaybeMoved` として保持する。
- `availability_state` は raw cell の exact/dynamic alias 関係でも non-initialized state を参照し、unknown-offset `MaybeMoved` を後続の load / dealloc が見落とさない。
- `resource_ir_cell_check_unknown_offset_non_copy_move_keeps_dealloc_conservative` を追加し、dynamic non-Copy store/load 後の storage-only dealloc が `RawMemoryDeallocCell` / `MaybeMoved` で拒否されることを固定した。

## 検証結果

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_unknown_offset_non_copy_move_keeps_dealloc_conservative -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_tracks_external_non_copy_raw_load_after_first_move -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_raw_load_before_store -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill_does_not_initialize_non_copy_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_external_fd_read_initializes_iovec_buffers -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read_reports_uninitialized_iovec_descriptor -- --nocapture`: passed
- `resource_ir_cell_check_keeps_unknown_arithmetic_helper_offset_conservative` と `resource_ir_cell_check_preserves_mem_ptr_disjoint_offsets` は、既存の `ShadowSameSignatureCallable` warning を `typecheck_resource_source` helper が失敗扱いするため未完了。今回の Resource IR cell state 変更に到達する前の既存 test harness 問題として扱う。
