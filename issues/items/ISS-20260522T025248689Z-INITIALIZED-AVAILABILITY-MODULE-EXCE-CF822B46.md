---
id: ISS-20260522T025248689Z-INITIALIZED-AVAILABILITY-MODULE-EXCE-CF822B46
title: "initialized availability module exceeds responsibility limit"
area: core
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_availability.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T025248689Z-INITIALIZED-AVAILABILITY-MODULE-EXCE-CF822B46: initialized availability module exceeds responsibility limit

## 概要

After splitting collection_slot_state_release_alias.rs, the resource responsibility monitor reaches initialized_availability.rs and fails because the module has 173 lines while its split limit is 120. The module mixes argument availability orchestration, by-value consumption, unavailable diagnostics, and collection-slot certified raw-cell acceptance helpers.

## 対象

- `nepl-core/src/resource/initialized_availability.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting collection_slot_state_release_alias.rs, the resource responsibility monitor reaches initialized_availability.rs and fails because the module has 173 lines while its split limit is 120. The module mixes argument availability orchestration, by-value consumption, unavailable diagnostics, and collection-slot certified raw-cell acceptance helpers.

## 影響

The resource checker responsibility gate remains blocked after the collection-slot release alias split, so future static-check changes lose an automated signal for initialized availability module growth.

## 修正方針

Split initialized_availability.rs into focused modules for argument availability/consumption and collection-slot certified raw-cell acceptance, without raising the limit.

## 対応内容

`initialized_availability.rs` は raw cell availability / argument availability / by-value consumption の基本処理に絞った。collection slot state によって raw realloc 前の live non-Copy raw cells を証明する処理は `initialized_availability_collection.rs` に分離し、CellUnavailable diagnostic emission は `initialized_availability_diagnostic.rs` に分離した。

この分割により、source 由来の collection slot proof と通常の cell availability check を同じ module に混在させず、Resource IR 静的検査の責務境界を維持できる。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and issue validation.

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_realloc_rekeys_collection_managed_non_copy_raw_cell -- --test-threads=1 --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_relocate_accepts_live_non_copy_payload_after_realloc -- --test-threads=1 --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_raw_dealloc -- --test-threads=1 --nocapture`: pass
- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_availability.rs` blocker は解消。次の既存 blocker として `initialized_raw_memory.rs has 198 lines; responsibility split limit is 190` を検出したため、`ISS-20260522T030231180Z-INITIALIZED-RAW-MEMORY-DISPATCHER-EX-3FB72BA0` に分離した。
