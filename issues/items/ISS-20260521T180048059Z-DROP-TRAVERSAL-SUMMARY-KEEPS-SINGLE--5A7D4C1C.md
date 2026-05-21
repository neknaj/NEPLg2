---
id: ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C
title: "Drop traversal summary keeps single-variant coverage wrapper after forall rollback"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_model.rs
---

# ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C: Drop traversal summary keeps single-variant coverage wrapper after forall rollback

## 概要

After closing the unsound ForallInitializedRange producer, CollectionSlotLifecycleSummaryDropTraversalCoverage has only CertifiedSlots. Keeping a coverage enum with one production mode makes the Resource IR summary model look more general than it is and invites future code to re-add broad full-range behavior without a typed certificate payload.

## 対象

- `nepl-core/src/resource/collection_slot_summary_model.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、Resource IR の proof boundary を audit しやすい enum / struct / match で管理し、証明できない状態を型の上に残さないことを要求している。
- [ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7](./ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7.md) で unsound な `ForallInitializedRange` producer と replay variant を削除したため、`CollectionSlotLifecycleSummaryDropTraversalCoverage` は単一 variant の wrapper になっていた。
- [ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF](./ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF.md) の full-range certificate はまだ open であり、source loop / iterator / traversal coverage を表す typed certificate が導入されるまでは full-range 証明 mode を実装上に残してはいけない。

## 問題

After closing the unsound ForallInitializedRange producer, CollectionSlotLifecycleSummaryDropTraversalCoverage has only CertifiedSlots. Keeping a coverage enum with one production mode makes the Resource IR summary model look more general than it is and invites future code to re-add broad full-range behavior without a typed certificate payload.

## 影響

This leaves avoidable complexity in the static-check proof surface. It weakens auditability because callers must still match through a proof-mode enum even though the only sound mode is finite certified slots.

## 修正方針

Replace the degenerate coverage enum with an explicit certified_slots field on DropTraversal summaries. Reintroduce a full-range variant only together with a source traversal coverage certificate model.

## 検証

- `CollectionSlotLifecycleSummaryOp::DropTraversal` は `coverage` wrapper ではなく `certified_slots` field を直接持つ。
- summary build / translate / replay は finite certified slot payload だけを扱い、full-range cleanup を示す mode は存在しない。
- full-range summary を再導入する場合は、`ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF` の source traversal coverage certificate と同時に入れる必要がある。
- 実行結果:
  - `cargo test -p nepl-core --lib collection_slot_summary_build_ops -- --test-threads=1`: pass
  - `cargo check -p nepl-core`: pass
  - `cargo fmt --check -p nepl-core`: pass
  - `node nodesrc/test_resource_checker_responsibility.js`: pass
  - `node nodesrc/issues.js check --dir issues`: pass
  - `git diff --check`: pass（CRLF warning のみ）
