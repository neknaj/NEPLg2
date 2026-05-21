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
- [ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C](./ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C.md) で、full-range producer を持たない単一 variant wrapper を削除し、現状の summary は finite `certified_slots` だけを明示する形へ戻した。
- 既存の finite `certified_slots` replay は、callee 内で列挙された slot だけを dropped にするため、caller 側に同じ storage prefix かつ `initialized_count` 内の別 initialized slot がある場合に generic traversal cleanup を表現できない。
- `ResourceOp::Loop` と `ResourceConditionFact` の path-local fact だけを full initialized-range proof として扱うのは不十分だった。今回の修正では、`i = 0; i < initialized_count; i += 1` の loop induction、全域 coverage、body 内 exact slot load/drop witness、storage/count/index の不変性を typed certificate の生成条件にした。

## 問題

CollectionSlotDropTraversal summaries replay only the finite callee certified_slots list. That proves particular slots were dropped, but it does not prove every caller initialized slot covered by storage and initialized_count is dropped. Non-Copy collection cleanup therefore cannot be certified generically across helper calls.

## 影響

Self-host collections with dynamic initialized ranges would either keep rejecting valid cleanup or require stdlib/module-specific allowlists, which violates the Resource IR generic proof design and leaves memory-safety proof debt.

## 修正方針

Introduce a typed forall initialized-range summary proof, derive it from source Resource IR drop traversal range facts, and replay it by validating all caller initialized slots under the storage against initialized_count before applying summary-certified loaded-value drops.

## 進捗

- `CollectionSlotLifecycleSummaryOp::DropTraversal` は `CollectionSlotLifecycleSummaryDropTraversalCoverage::{CertifiedSlots, ForallInitializedRange}` を持つ。finite slot certificate は従来どおり replay し、full-range cleanup は typed certificate がある場合だけ replay する。
- follow-up の [ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7](./ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7.md) で、per-slot symbolic/range witness から full initialized-range summary を生成する経路は閉じた。
- [ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C](./ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C.md) で、dead code の typed replay mode を削除した後に残った単一 variant wrapper も削除した。今回の修正では、その反省を踏まえて producer / summary payload / replay / negative regression を同時に揃えた。
- marker-only helper は引き続き summary を生成しない。
- `CollectionSlotInitializedRangeDropTraversalCertificate` は element stride と drop obligation を保持する typed struct であり、summary build / translate / replay は enum variant の exhaustive match で分岐する。文字列、bool sentinel、stdlib module allowlist は追加していない。
- summary build は、loop condition の `index < initialized_count`、zero-based start、strict one-step increment、body prefix の raw load / actual drop witness、body 全体の storage/count preservation、increment 後の index preservation をすべて満たす場合だけ `ForallInitializedRange` を生成する。
- caller replay は certificate の element stride を `storage_size_bytes(expected_ty)` と照合し、既存の collection slot range traversal checker に `SummaryCertified(DropLoadedValue)` proof を渡して、caller storage prefix 配下かつ initialized_count 内の initialized slot 全体を検査する。

## 検証

- `collection_slot_summary_loop_induction_certifies_forall_drop_traversal`
- `collection_slot_summary_loop_induction_rejects_tail_storage_mutation`
- `collection_slot_summary_forall_replay_drops_every_initialized_slot_in_range`
- `collection_slot_summary_build_ops_tests::collection_slot_summary_branch_condition_fact_does_not_certify_forall_drop_traversal`
- `resource_ir_collection_slot_drop_traversal_summary_rejects_marker_only_cleanup`
- `resource_ir_collection_slot_drop_traversal_accepts_symbolic_slot_with_range_proof`
