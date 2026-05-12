---
id: ISS-20260512T140149223Z-RESOURCE-IR-DUMP-DUPLICATES-RAWMEMOR-61AE6F81
title: "Resource IR dump duplicates RawMemoryOp string mapping"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/dump.rs; nepl-core/src/effects.rs"
---

# ISS-20260512T140149223Z-RESOURCE-IR-DUMP-DUPLICATES-RAWMEMOR-61AE6F81: Resource IR dump duplicates RawMemoryOp string mapping

## 概要

Resource IR dump has a local dump_raw_memory_op match that repeats RawMemoryOp textual names already owned by RawMemoryOp::as_str and Display.

## 対象

- `nepl-core/src/resource/dump.rs; nepl-core/src/effects.rs`

## 根拠

- `nepl-core/src/effects.rs` は `RawMemoryOp::as_str` と `Display` を既に提供している。
- `nepl-core/src/resource/dump.rs` は別途 `dump_raw_memory_op` を持ち、`RawMemoryOp` の全 variant を同じ文字列へ再マッピングしていた。
- Resource IR dump は static-check / effect boundary の監査入力なので、operation 名の authority が複数あると Stage 5 の enum-first 方針が崩れる。

## 問題

Resource IR dump has a local dump_raw_memory_op match that repeats RawMemoryOp textual names already owned by RawMemoryOp::as_str and Display.

## 影響

Raw memory operation naming can diverge between effect diagnostics, Resource IR dump snapshots, and downstream review tools when a RawMemoryOp variant is added or renamed.

## 修正方針

Remove the Resource IR dump-local RawMemoryOp string mapping and use RawMemoryOp Display as the single operation-name authority.

## 対応記録

- `ResourceOp::RawMemory` の dump 出力で `RawMemoryOp` の `Display` を直接使うようにした。
- `dump_raw_memory_op` を削除し、Resource IR dump が raw operation 名を独自に再マッピングしないようにした。
- `nodesrc/test_resource_checker_responsibility.js` に `dump_raw_memory_op` 再導入を拒否する source policy を追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_raw_memory_operations -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
