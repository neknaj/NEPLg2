---
id: ISS-20260517T053938813Z-RAW-MEMORY-INTRINSIC-EFFECT-STILL-US-63EAEAFB
title: "raw memory intrinsic effect still uses string marker list"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/effects.rs
---

# ISS-20260517T053938813Z-RAW-MEMORY-INTRINSIC-EFFECT-STILL-US-63EAEAFB: raw memory intrinsic effect still uses string marker list

## 概要

effects.rs still keeps RAW_MEMORY_INTRINSIC_EFFECT_MARKERS for #intrinsic load/store and typecheck/effect_check.rs combines that string marker with raw_memory_op_from_name. The raw memory intrinsic effect gate therefore depends on a duplicated string list instead of a single typed RawMemoryOp classifier.

## 対象

- `nepl-core/src/effects.rs`

## 根拠

- `effects.rs` は `#intrinsic "load"` / `"store"` の raw memory effect 判定を `RAW_MEMORY_INTRINSIC_EFFECT_MARKERS` という string marker list に保持していた。
- 一方で actual operation は `raw_memory_op_from_name` が `RawMemoryOp::Load` / `Store` として分類しており、`typecheck/effect_check.rs` は marker list と operation 再分類を組み合わせていた。
- これでは intrinsic spelling と typed operation の対応が 2 箇所に分散し、pure context gate が enum/match ではなく string list agreement に依存する。

## 問題

effects.rs still keeps RAW_MEMORY_INTRINSIC_EFFECT_MARKERS for #intrinsic load/store and typecheck/effect_check.rs combines that string marker with raw_memory_op_from_name. The raw memory intrinsic effect gate therefore depends on a duplicated string list instead of a single typed RawMemoryOp classifier.

## 影響

If load/store intrinsic spelling or raw-memory operation classification changes, the marker list and RawMemoryOp mapping can diverge. Static-check pure-context enforcement would then depend on string list agreement instead of enum/match exhaustiveness.

## 修正方針

- `RAW_MEMORY_INTRINSIC_EFFECT_MARKERS` を削除し、`raw_memory_intrinsic_op_from_name` で `load` / `store` を `RawMemoryOp::Load` / `Store` として返す。
- `intrinsic_is_raw_memory_effect` と `BlockChecker::raw_memory_intrinsic_allowed` は marker list ではなく typed `RawMemoryOp` classifier を直接消費する。
- tests と source policy で marker list の再導入と consumer 側の marker + reclassification 構造への退行を拒否する。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test effects raw_memory_intrinsic -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test effects raw_memory -- --nocapture`: 17 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
