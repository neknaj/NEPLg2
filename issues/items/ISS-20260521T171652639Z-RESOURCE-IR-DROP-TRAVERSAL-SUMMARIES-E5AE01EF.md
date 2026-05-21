---
id: ISS-20260521T171652639Z-RESOURCE-IR-DROP-TRAVERSAL-SUMMARIES-E5AE01EF
title: "Resource IR drop traversal summaries need typed forall initialized-range certificates"
area: core
status: open
resolved: false
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
- 現行 `ResourceOp::Loop` と `ResourceConditionFact` は path-local な condition fact と body/exit merge を表すだけで、`i = 0; i < initialized_count; i += 1` のような loop induction、全域 coverage、body 内 exact slot load/drop witness、storage/count/index の不変性を typed certificate として持たない。したがって per-slot symbolic range proof を full initialized-range proof へ昇格してはいけない。

## 問題

CollectionSlotDropTraversal summaries replay only the finite callee certified_slots list. That proves particular slots were dropped, but it does not prove every caller initialized slot covered by storage and initialized_count is dropped. Non-Copy collection cleanup therefore cannot be certified generically across helper calls.

## 影響

Self-host collections with dynamic initialized ranges would either keep rejecting valid cleanup or require stdlib/module-specific allowlists, which violates the Resource IR generic proof design and leaves memory-safety proof debt.

## 修正方針

Introduce a typed forall initialized-range summary proof, derive it from source Resource IR drop traversal range facts, and replay it by validating all caller initialized slots under the storage against initialized_count before applying summary-certified loaded-value drops.

## 進捗

- `CollectionSlotLifecycleSummaryOp::DropTraversal` は `certified_slots` field だけを持つ。finite slot certificate は従来どおり replay する。
- follow-up の [ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7](./ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7.md) で、per-slot symbolic/range witness から full initialized-range summary を生成する経路は閉じた。
- [ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C](./ISS-20260521T180048059Z-DROP-TRAVERSAL-SUMMARY-KEEPS-SINGLE--5A7D4C1C.md) で、dead code の typed replay mode を削除した後に残った単一 variant wrapper も削除した。full initialized-range summary は、source traversal coverage の typed certificate を導入する本 issue の残件として扱う。
- marker-only helper は引き続き summary を生成しない。
- replay は文字列や bool sentinel による証明モード管理を追加していない。full-range mode を再導入する場合は、`InitializedRangeDropTraversalCertificate` のような typed struct と summary variant を同じ変更で追加し、producer / replay / negative regression を同時に揃える。

## 検証

- `collection_slot_summary_build_ops_tests::collection_slot_summary_branch_condition_fact_does_not_certify_forall_drop_traversal`
- `resource_ir_collection_slot_drop_traversal_summary_rejects_marker_only_cleanup`
- `resource_ir_collection_slot_drop_traversal_accepts_symbolic_slot_with_range_proof`
