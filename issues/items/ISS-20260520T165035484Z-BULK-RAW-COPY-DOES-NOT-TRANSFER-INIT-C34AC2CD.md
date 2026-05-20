---
id: ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD
title: "Bulk raw copy does not transfer initialized raw range evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260520T165035484Z-BULK-RAW-COPY-DOES-NOT-TRANSFER-INIT-C34AC2CD: Bulk raw copy does not transfer initialized raw range evidence

## 概要

RawMemoryOp::BulkCopy/BulkMove lifecycle handling copies initialized Copy cell entries but does not transfer initialized raw byte/element range evidence. A raw range initialized by fill or external proof can therefore be lost across a byte copy even when extent proof should preserve it.

## 対象

- `nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/cell_state_raw_range.rs, nepl-core/tests/resource_ir.rs`

## 根拠

`RawCellLifecycleEvent` の bulk copy/move path は source/destination だけを受け取り、`len` による byte extent proof を lifecycle transition に渡していなかった。そのため fixed initialized Copy cell は転送できても、`fill_u8` / `fill_i32` が作る `InitializedRawByteRange` を destination 側へ再投影できなかった。

## 問題

RawMemoryOp::BulkCopy/BulkMove lifecycle handling copies initialized Copy cell entries but does not transfer initialized raw byte/element range evidence. A raw range initialized by fill or external proof can therefore be lost across a byte copy even when extent proof should preserve it.

## 影響

Buffer and collection code that uses raw copy as a storage operation cannot rely on Resource IR to preserve initialized prefix facts. Fixing this with stdlib allowlists would violate the generic proof-boundary design; leaving it unfixed causes false positives or pressures weakening RawMemoryLoadCell.

## 修正方針

Extend the typed raw lifecycle transition model so bulk copy/move consumes an explicit extent/count proof and transfers only range evidence covered by that proof, or emits a precise diagnostic when range transfer cannot be proven.

## 検証

Add Resource IR regressions for initialized byte ranges and element ranges crossing bulk copy/move, plus negative tests where count/extent proof is missing.

## 2026-05-21 修正

bulk copy/move の raw lifecycle event を `BulkCopyInitializedRawState` として再設計し、copy length を transition に渡すようにした。これにより、destination 側へ転送する initialized state は `len` で証明できる範囲だけに限定される。

修正内容:

- `RawMemoryOp::BulkCopy` / `BulkMove` で第 3 引数の byte count を scalar fact として canonicalize し、未初期化 count を `RawMemoryBulkCount` として検出する。
- `CellTable::copy_initialized_copy_raw_cells_covered_by_count` を追加し、Copy raw cell も byte count が cell 全体を覆う場合だけ転送する。
- `CellTable::copy_initialized_raw_byte_ranges_for_bulk_copy` を追加し、byte range / element range を copy count proof に基づいて destination へ再投影する。
- `i32_extent_proof` を追加し、scalar count と scaled count の被覆証明を bulk copy 専用の ad hoc 判定ではなく共有 helper として分離した。
- copy count が source の byte range 全体を覆う場合は元の count/unit/type を保持し、prefix だけを覆う場合は安全に証明できる prefix count だけを destination range として記録する。
- stdlib helper 名の allowlist や個別関数例外は追加していない。Resource IR の `RawCellAddressAliases` が保持する定数・関係・scale fact に基づいて証明する。

追加した回帰:

- `resource_ir_cell_check_bulk_copy_transfers_initialized_byte_ranges`
- `resource_ir_cell_check_bulk_copy_transfers_initialized_copy_cells_with_extent`
- `resource_ir_cell_check_bulk_move_transfers_initialized_element_ranges`
- `resource_ir_cell_check_bulk_copy_does_not_transfer_uncovered_byte_ranges`

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_bulk -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_initialized -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_bulk_copy_of_live_non_copy_raw_cells -- --test-threads=1 --exact`: passed
