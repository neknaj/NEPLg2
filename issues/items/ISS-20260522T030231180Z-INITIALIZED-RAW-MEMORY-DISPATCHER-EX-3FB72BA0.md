---
id: ISS-20260522T030231180Z-INITIALIZED-RAW-MEMORY-DISPATCHER-EX-3FB72BA0
title: "initialized raw memory dispatcher exceeds responsibility limit again"
area: core
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T030231180Z-INITIALIZED-RAW-MEMORY-DISPATCHER-EX-3FB72BA0: initialized raw memory dispatcher exceeds responsibility limit again

## 概要

After splitting initialized_availability.rs, the resource responsibility monitor reaches initialized_raw_memory.rs and fails because the dispatcher has 198 lines while its split limit is 190. The module still mixes RawMemoryOp dispatch with raw dealloc collection-slot refutation reporting and realloc/dealloc bookkeeping.

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting initialized_availability.rs, the resource responsibility monitor reaches initialized_raw_memory.rs and fails because the dispatcher has 198 lines while its split limit is 190. The module still mixes RawMemoryOp dispatch with raw dealloc collection-slot refutation reporting and realloc/dealloc bookkeeping.

## 影響

The resource checker responsibility gate remains blocked after the initialized availability split, and raw-memory static-check changes can keep accumulating in the dispatcher instead of focused proof modules.

## 修正方針

Split initialized_raw_memory.rs so raw dealloc collection-slot release/refutation reporting and any remaining realloc/dealloc bookkeeping have focused modules, without increasing the dispatcher limit.

## 対応内容

`initialized_raw_memory.rs` は `RawMemoryOp` dispatcher と raw memory operation routing に集中させた。raw dealloc 時に collection slot state を alias-aware に release し、refutation を `CollectionSlotRefuted` diagnostic へ変換する処理は `initialized_raw_memory_dealloc_collection.rs` に分離した。

この分割により、raw dealloc の collection-slot proof / diagnostic emission と dispatcher 本体を別責務として監査できる。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and raw-memory focused resource_ir tests.

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_raw_dealloc -- --test-threads=1 --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_dealloc -- --test-threads=1 --nocapture`: pass
- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_raw_memory.rs` blocker は解消。次の既存 blocker として `initialized_summary.rs has 81 lines; responsibility split limit is 80` を検出したため、`ISS-20260522T031252045Z-INITIALIZED-SUMMARY-MODEL-EXCEEDS-RE-C686AE30` に分離した。
