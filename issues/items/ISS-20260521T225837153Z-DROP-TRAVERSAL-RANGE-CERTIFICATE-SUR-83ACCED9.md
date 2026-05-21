---
id: ISS-20260521T225837153Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-SUR-83ACCED9
title: "Drop traversal range certificate survives raw load from protected storage"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_range_certificate.rs, nepl-core/src/resource/collection_slot_summary_build_range_lifetime.rs, nepl-core/src/resource/collection_slot_summary_build_range_preserve*.rs"
---

# ISS-20260521T225837153Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-SUR-83ACCED9: Drop traversal range certificate survives raw load from protected storage

## 概要

ForallInitializedRange certificates can survive a RawMemory::Load from the protected storage range after the certified drop witness, because preservation checks treat raw loads as anchor-preserving even though typed raw loads can move non-Copy slot state.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_range_certificate.rs, nepl-core/src/resource/collection_slot_summary_build_range_lifetime.rs, nepl-core/src/resource/collection_slot_summary_build_range_preserve*.rs`

## 根拠

- Parent issue: [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 設計段階: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成)
- 直前の関連修正: [ISS-20260521T223133295Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-E9B96873](./ISS-20260521T223133295Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-E9B96873.md)
- 監査観点: `ForallInitializedRange` certificate は stdlib helper allowlist ではなく、Resource IR の loop induction、actual drop witness、alias-aware preservation、post-loop lifetime による generic proof として扱う必要がある。

## 問題

ForallInitializedRange certificates can survive a RawMemory::Load from the protected storage range after the certified drop witness, because preservation checks treat raw loads as anchor-preserving even though typed raw loads can move non-Copy slot state.

## 影響

A callee summary may claim full initialized-range cleanup even after the loop tail or post-loop code has moved a protected collection slot again, weakening Resource IR memory-safety proof for non-Copy collection cleanup.

## 修正方針

Differentiate the certified witness prefix from later code: allow the proof witness raw load/drop, but reject protected-storage raw loads in the loop tail and invalidate existing certificates when post-loop raw loads touch protected storage.

## 対応

- loop-body candidate を storage / type / stride だけではなく、witness load index と direct `ResourceOp::Drop` index を持つ `LoopBodyDropWitnessCandidate` として扱うようにした。
- 通常の `body_preserves_place` は actual drop witness の `RawMemoryOp::Load` を許すため維持し、witness 付き body 用に `body_preserves_place_with_drop_witness`、witness 後 tail 用に `body_preserves_place_after_drop_witness` / `op_preserves_place_after_drop_witness` を追加した。
- witness 付き preservation は alias propagation を反映しながら、選択された witness load 以外の protected storage typed load を拒否する。これにより witness drop 後かつ induction step 前の追加 raw load / unsafe-memory load call も full-range certificate を維持できない。
- tail 用 preservation は induction step 後の protected storage typed load も拒否する。
- post-loop lifetime 側も `RawMemoryOp::Load` の args が certificate storage に触れる場合に certificate を失効させるようにした。`LoadU8` / memory size / grow は slot owner state を move しないため、既存の preserve 分類を維持する。
- 実装は `RawMemoryOp` enum と `ResourceOp` match の分岐で行い、stdlib module 名や helper 名の allowlist は追加していない。

## 検証

Add loop-tail and post-loop raw-load regressions, then run focused range certificate/lifetime tests plus cargo check and responsibility monitor.

- `cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --nocapture`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --nocapture`
- `cargo test -p nepl-core --lib body_preserve -- --nocapture`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
