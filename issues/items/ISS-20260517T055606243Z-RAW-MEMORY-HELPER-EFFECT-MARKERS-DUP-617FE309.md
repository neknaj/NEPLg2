---
id: ISS-20260517T055606243Z-RAW-MEMORY-HELPER-EFFECT-MARKERS-DUP-617FE309
title: "raw memory helper effect markers duplicate typed operation mapping"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/effects.rs
---

# ISS-20260517T055606243Z-RAW-MEMORY-HELPER-EFFECT-MARKERS-DUP-617FE309: raw memory helper effect markers duplicate typed operation mapping

## 概要

effects.rs still keeps RAW_MEMORY_HELPER_EFFECT_MARKERS as a string list and then raw_memory_op_from_name repeats the same helper spelling to RawMemoryOp mapping. Raw memory helper effect classification therefore depends on a marker list plus a separate match instead of a typed helper domain.

## 対象

- `nepl-core/src/effects.rs`

## 根拠

- `effects.rs` は raw memory helper spelling 群を `RAW_MEMORY_HELPER_EFFECT_MARKERS` string list に保持していた。
- 同じ file の `raw_memory_op_from_name` は、その marker list で既知名かどうかを確認した後、別の `match` で同じ spelling を `RawMemoryOp` に再分類していた。
- `raw_memory_op_from_name` は source capability proof、Resource IR lowering、effect gate から参照される中核 classifier であり、ここが marker list と match の二重管理だと checker 自体の誤りを Rust の型で検出しにくい。

## 問題

effects.rs still keeps RAW_MEMORY_HELPER_EFFECT_MARKERS as a string list and then raw_memory_op_from_name repeats the same helper spelling to RawMemoryOp mapping. Raw memory helper effect classification therefore depends on a marker list plus a separate match instead of a typed helper domain.

## 影響

Raw memory helper spellings can drift from the RawMemoryOp mapping. Source capability, Resource IR, and effect gates depend on this classifier, so duplicated string lists weaken static-check correctness and make checker mistakes harder to catch.

## 修正方針

Replace RAW_MEMORY_HELPER_EFFECT_MARKERS with a typed RawMemoryHelper enum that owns helper spelling and RawMemoryOp mapping. raw_memory_op_from_name should classify through that enum, and tests/source policy should reject reintroducing the string marker list.

## 対応内容

- `RAW_MEMORY_HELPER_EFFECT_MARKERS` と `raw_memory_base_is_known` を削除した。
- `RawMemoryHelper` enum を追加し、runtime ABI helper、core/mem raw helper、load/store/fill/bulk helper の base spelling と `RawMemoryOp` mapping を `base_name()` / `operation()` / `from_name()` に集約した。
- `raw_memory_op_from_name` は `RawMemoryHelper::from_name(name).map(RawMemoryHelper::operation)` だけを消費するようにした。
- effects test と source policy を更新し、helper domain の round-trip と旧 marker list の再導入禁止を固定した。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test effects raw_memory_helper -- --nocapture`: 4 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
