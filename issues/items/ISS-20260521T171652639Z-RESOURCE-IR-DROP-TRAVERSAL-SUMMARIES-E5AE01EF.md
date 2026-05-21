---
id: ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF
title: "Resource IR drop traversal summaries need typed forall initialized-range certificates"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_drop_traversal*.rs, nepl-core/src/resource/collection_slot_summary_*.rs"
---

# ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF: Resource IR drop traversal summaries need typed forall initialized-range certificates

## 概要

CollectionSlotDropTraversal summaries replay only the finite callee certified_slots list. That proves particular slots were dropped, but it does not prove every caller initialized slot covered by storage and initialized_count is dropped. Non-Copy collection cleanup therefore cannot be certified generically across helper calls.

## 対象

- `nepl-core/src/resource/collection_slot_drop_traversal*.rs, nepl-core/src/resource/collection_slot_summary_*.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection slot lifecycle / drop traversal を stdlib module allowlist ではなく Resource IR の generic proof boundary として扱うことを完了条件にしている。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload cleanup を compiler-issued owner token / InitializedCell / Resource IR state へ載せる親 issue である。
- 既存の finite `certified_slots` replay は、callee 内で列挙された slot だけを dropped にするため、caller 側に同じ storage prefix かつ `initialized_count` 内の別 initialized slot がある場合に generic traversal cleanup を表現できなかった。

## 問題

CollectionSlotDropTraversal summaries replay only the finite callee certified_slots list. That proves particular slots were dropped, but it does not prove every caller initialized slot covered by storage and initialized_count is dropped. Non-Copy collection cleanup therefore cannot be certified generically across helper calls.

## 影響

Self-host collections with dynamic initialized ranges would either keep rejecting valid cleanup or require stdlib/module-specific allowlists, which violates the Resource IR generic proof design and leaves memory-safety proof debt.

## 修正方針

Introduce a typed forall initialized-range summary proof, derive it from source Resource IR drop traversal range facts, and replay it by validating all caller initialized slots under the storage against initialized_count before applying summary-certified loaded-value drops.

## 修正内容

- `CollectionSlotLifecycleSummaryOp::DropTraversal` の `certified_slots + proof` を `CollectionSlotLifecycleSummaryDropTraversalCoverage` enum に置き換えた。
- `CertifiedSlots(Vec<...>)` は従来どおり finite slot certificate として replay し、`ForallInitializedRange` は caller 側の storage prefix 配下にある initialized slot 全体を `initialized_count` と element stride で検証してから summary-certified loaded-value drop を適用する。
- summary build は source-derived symbolic/range slot witness を持つ traversal だけを `ForallInitializedRange` にし、marker-only helper は引き続き summary を生成しない。
- replay は enum の match で分岐し、文字列や bool sentinel による証明モード管理を追加していない。

## 検証

- `collection_slot_summary_forall_drop_tests::forall_drop_summary_replay_drops_every_caller_slot_inside_count`
- `collection_slot_summary_forall_drop_tests::forall_drop_summary_replay_rejects_caller_slot_outside_count`
- `collection_slot_summary_build_ops_tests::collection_slot_summary_branch_condition_fact_certifies_symbolic_drop_traversal`
- `resource_ir_collection_slot_drop_traversal_summary_rejects_marker_only_cleanup`
- `resource_ir_collection_slot_drop_traversal_accepts_symbolic_slot_with_range_proof`
