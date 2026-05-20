---
id: ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2
title: "Collection slot lifecycle state does not transfer across storage relocation"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/**, nepl-core/src/resource_primitives/**, nepl-core/src/typecheck/**"
---

# ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2: Collection slot lifecycle state does not transfer across storage relocation

## 概要

CollectionSlotStateTable can prove slot initialize/move/drop/release on a single storage place, but it has no generic transition for realloc/grow style storage owner relocation. After backing storage moves from old owner/place to new owner/place, initialized/moved/dropped slot facts remain keyed to the old prefix, so non-Copy collection payload support would need Vec-specific proof or unsafe source convention.

## 対象

- `nepl-core/src/resource/**, nepl-core/src/resource_primitives/**, nepl-core/src/typecheck/**`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection payload を stdlib module allowlist ではなく Resource IR の generic typed proof boundary に載せる方針を明記している。
- 既存の `ResourceOp::CollectionSlotLifecycle` は単一 storage prefix 上の slot lifecycle を追跡できたが、grow / realloc / storage owner replacement で old storage prefix から new storage prefix へ state を移す Resource IR transition がなかった。
- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の non-Copy `Vec<T>` / `OwnedBuffer<T>` 実装では、storage relocation 後も initialized / moved / dropped slot state が保持される必要がある。

## 問題

CollectionSlotStateTable can prove slot initialize/move/drop/release on a single storage place, but it has no generic transition for realloc/grow style storage owner relocation. After backing storage moves from old owner/place to new owner/place, initialized/moved/dropped slot facts remain keyed to the old prefix, so non-Copy collection payload support would need Vec-specific proof or unsafe source convention.

## 影響

Non-Copy Vec/OwnedBuffer grow cannot be proven by compiler core. If stdlib removes Copy-only restrictions before this exists, live owner payloads can be lost, double-dropped, or accepted through module-specific annotations instead of the generic Resource IR proof boundary.

## 修正方針

Add a typed collection storage relocation primitive and Resource IR transition that rekeys all slot states covered by the old storage prefix to the new storage prefix. Integrate it with control-flow merge and call summaries without stdlib function-name allowlists.

## 修正結果

- `CollectionSlotLifecyclePrimitive::StorageRelocate` と `ResourceOp::CollectionStorageRelocate` を追加し、old / new storage pair を typed Resource IR operation として表すようにした。
- `CollectionSlotStateTable::relocate_storage` を専用 module に分離し、old storage 配下の initialized / moved / dropped slot state を new storage prefix へ rekey する。old storage は released として閉じ、new storage 側に既存 slot state がある場合は typed refutation で拒否する。
- HIR lowering、typecheck prefix check、Resource IR dump / coverage / borrow usage / initialized check / summary build / summary apply へ `CollectionStorageRelocate` を exhaustive `match` で接続した。
- call summary は `Relocate { old_storage, new_storage }` を parameter-relative target として保持し、caller の actual storage pair へ instantiation して replay する。
- `collection_slot_state_relocate.rs`、`collection_slot_state_relocate_tests.rs`、`lower_temporary_scope_op.rs` へ責務を分離し、既存の resource checker line-limit policy を緩めずに実装した。

## 検証

- `cargo test -p nepl-core storage_relocate -- --test-threads=1`: pass
- `cargo fmt --check -p nepl-core`: pass
- `cargo check -p nepl-core`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
