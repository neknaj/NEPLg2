---
id: ISS-20260518T112010000Z-RESOURCE-LOWERING-RAW-MEMORY-INTRINS-A25124A3
title: "Resource lowering raw memory intrinsic classifier still uses boolean gate"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/lower_raw_memory.rs, nepl-core/src/effects.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260518T112010000Z-RESOURCE-LOWERING-RAW-MEMORY-INTRINS-A25124A3: Resource lowering raw memory intrinsic classifier still uses boolean gate

## 概要

Resource lowering classifies raw memory intrinsics by first asking intrinsic_is_raw_memory_effect and then reclassifying through raw_memory_op_from_name. Typecheck already consumes raw_memory_intrinsic_op_from_name directly, so Resource lowering keeps a boolean marker-style gate that can drift from the typed intrinsic operation classifier.

## 対象

- `nepl-core/src/resource/lower_raw_memory.rs, nepl-core/src/effects.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `typecheck/effect_check.rs` は `raw_memory_intrinsic_op_from_name` を直接消費し、`#intrinsic "load"` / `"store"` を `RawMemoryOp` として扱う。
- 一方で `resource/lower_raw_memory.rs` は `intrinsic_is_raw_memory_effect(name)` で boolean 判定した後、`raw_memory_op_from_name(name)` で helper 名 classifier に再分類していた。
- この経路では raw intrinsic classifier と helper classifier の 2 箇所が一致していることに依存し、Resource IR lowering 側の静的検査入力が typecheck と drift し得る。

## 問題

Resource lowering classifies raw memory intrinsics by first asking intrinsic_is_raw_memory_effect and then reclassifying through raw_memory_op_from_name. Typecheck already consumes raw_memory_intrinsic_op_from_name directly, so Resource lowering keeps a boolean marker-style gate that can drift from the typed intrinsic operation classifier.

## 影響

If a future raw memory intrinsic is added to the typed classifier but not mirrored by the helper-name classifier, Resource IR lowering and coverage can silently miss the raw memory operation even though effect checking sees it. Static checking then depends on duplicate classification paths.

## 修正方針

Make Resource lowering consume raw_memory_intrinsic_op_from_name directly and add a source policy regression that forbids intrinsic_is_raw_memory_effect in lower_raw_memory.rs.

## 検証

Run cargo test -p nepl-core --test effects raw_memory_intrinsic, cargo check -p nepl-core --tests, Resource checker/source policy tests, issue check, and diff check.

## 解決内容

- `lower_raw_memory.rs` の raw intrinsic lowering を `raw_memory_intrinsic_op_from_name(name)` 直結にした。
- Resource lowering は `intrinsic_is_raw_memory_effect` の boolean gate と helper-name reclassification を組み合わせない。
- `nodesrc/test_resource_checker_responsibility.js` に source policy を追加し、`lower_raw_memory.rs` が typed raw intrinsic classifier を直接消費することと、boolean gate を再導入しないことを固定した。
