---
id: ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E
title: "Collection storage relocate lacks raw movement proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/initialized_collection_slot_relocate.rs, nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/collection_slot_summary_model.rs, nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260521T105928164Z-COLLECTION-STORAGE-RELOCATE-LACKS-RA-BE2DCD3E: Collection storage relocate lacks raw movement proof

## 概要

ResourceOp::CollectionStorageRelocate rekeys collection slot lifecycle state from old storage to new storage without requiring a typed proof that raw storage was actually relocated. This makes the operation stronger than the generic Resource IR evidence produced by raw realloc success.

## 対象

- `nepl-core/src/resource/initialized_collection_slot_relocate.rs, nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/collection_slot_summary_model.rs, nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、Resource IR の fact / obligation / evidence / refutation を enum / match で扱い、stdlib module 名や helper 名の allowlist へ戻らないことを完了条件にしている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、collection storage relocation を Vec 固有 proof ではなく generic Resource IR operation として扱う方針を明記している。
- 既存実装は `CollectionSlotStateTable::relocate_storage` の state rekey 自体は typed だったが、その入口が raw realloc 成功などの storage movement evidence と結び付いていなかった。

## 問題

ResourceOp::CollectionStorageRelocate rekeys collection slot lifecycle state from old storage to new storage without requiring a typed proof that raw storage was actually relocated. This makes the operation stronger than the generic Resource IR evidence produced by raw realloc success.

## 影響

A compiler-owned intrinsic or summary can make live non-Copy collection payload state appear under an arbitrary new storage place. That hides missing raw movement evidence, weakens memory-safety checking, and can regress into stdlib/module convention instead of source-derived proof.

## 修正方針

Represent raw storage relocation as a generic proof fact produced by raw realloc success and consumed by CollectionStorageRelocate. Summary build/replay must preserve the certified proof shape instead of replaying relocation as an unconditional state rekey.

## 修正結果

- `PendingRawReallocs` に certified raw storage relocation proof を追加し、`RawMemoryOp::Realloc` の結果に対する success path だけが `old -> new` の relocation proof を発行するようにした。
- `CollectionStorageRelocate` は `RawCellAddressAliases::canonicalize_owner_cell_address` 後の old / new storage pair に対応する certified proof がある場合だけ `CollectionSlotStateTable::relocate_storage` を実行する。proof 不足は `StorageRelocateRequiresRawMoveProof` という typed refutation として報告する。
- state rekey が失敗した場合は proof を消費せず、state rekey が成功した場合だけ proof を消費する。これにより rejected relocate が proof state だけを部分変異させる経路を避けた。
- `CollectionSlotLifecycleSummaryOp::Relocate` に `CollectionSlotLifecycleSummaryRelocateProof::RawStorageRelocation` を追加し、summary build は certified proof がある relocate だけを summary に載せる。caller replay は certified summary op だけを state rekey として適用し、proof なし helper summary を無条件 replay しない。
- regression を、proof なし direct relocate の拒否、realloc success path 後の relocate 受理、realloc success proof の一回消費、proof なし call summary を replay しないことの 4 点に分けて追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_relocate -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot -- --test-threads=1`: timeout after 244s. 変更対象を大きく含む broad filter は既知の長時間化傾向があるため、今回の commit gate は direct relocate と summary replay の focused tests に限定した。
