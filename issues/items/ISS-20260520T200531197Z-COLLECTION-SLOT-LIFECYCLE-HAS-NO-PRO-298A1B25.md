---
id: ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25
title: "Collection slot lifecycle has no production lowering producer"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/lower*.rs, nepl-core/src/resource/model.rs, nepl-core/src/source_capability/**, nepl-core/src/typecheck/**, nepl-core/src/resource_primitives/**, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T200531197Z-COLLECTION-SLOT-LIFECYCLE-HAS-NO-PRO-298A1B25: Collection slot lifecycle has no production lowering producer

## 概要

ResourceOp::CollectionSlotLifecycle is now checked by Resource IR, but no production lowering path emits it from real collection API semantics. Current regression coverage manually constructs the ResourceOp, so real Vec/OwnedBuffer operations cannot yet rely on the generic slot lifecycle proof.

## 対象

- `nepl-core/src/resource/lower*.rs, nepl-core/src/resource/model.rs, stdlib/alloc/collections/**, nepl-core/tests/resource_ir.rs`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection slot の live / moved / dropped / released state を stdlib module 名 allowlist ではなく compiler-core の generic typed proof boundary で検査する方針を示している。
- `ResourceOp::CollectionSlotLifecycle` と `CollectionSlotStateTable` は既に Resource IR checker 側へ接続済みだったが、実際の HIR/source からこの op を生成する compiler-owned producer がなかった。
- collection slot lifecycle boundary は `CollectionSlotLifecyclePrimitive` enum と `SourceCapabilityUseSite::CollectionSlotLifecycleBoundary` で表せるため、stdlib function name allowlist や module-specific proof machine を足さずに exact source evidence から生成できる。

## 問題

ResourceOp::CollectionSlotLifecycle is now checked by Resource IR, but no production lowering path emits it from real collection API semantics. Current regression coverage manually constructs the ResourceOp, so real Vec/OwnedBuffer operations cannot yet rely on the generic slot lifecycle proof.

## 影響

The compiler can prove slot lifecycle only for hand-written Resource IR. If stdlib non-Copy collection support proceeds before a typed lowering/annotation path exists, safety may fall back to module allowlists, inlining assumptions, or unchecked raw memory conventions.

## 修正方針

Design and implement a typed lowering source for collection slot lifecycle events. The producer must derive Initialize/BorrowRead/MoveOut/Replace/Drop/StorageDealloc events from source-level collection semantics or explicit compiler-owned annotations, not from stdlib function-name allowlists. It must feed ResourceOp::CollectionSlotLifecycle and preserve spans for diagnostics.

## 対応

- `CollectionSlotLifecyclePrimitive` を追加し、collection slot lifecycle boundary の種類、type arg 数、実引数数、slot offset の有無を typed enum で管理するようにした。
- source capability proof に `CollectionSlotLifecycleBoundary` を追加し、該当 intrinsic の exact span が compiler-proven stdlib source evidence を持つ場合だけ typecheck を通すようにした。
- typecheck は collection slot lifecycle intrinsic の source capability、type arg arity、argument arity、anchor が compiler memory pointer / owner token であること、slot offset が `i32` であることを検査する。
- slot lifecycle anchor は `MemPtr<T>` / `RegionToken<T>` の element type と intrinsic type args を照合する。replace event も old / new type args が storage element type と一致しなければ拒否し、typed proof state と実 raw storage 型がずれる経路を閉じた。
- `StorageDealloc` は owner token anchor に限定した。non-owning `MemPtr<T>` で storage release 証明を発行することはできない。
- Resource lowering は collection slot lifecycle intrinsic から `ResourceOp::CollectionSlotLifecycle` を生成し、slot event は raw base + byte offset、storage dealloc は storage anchor へ接続する。
- replace lowering の target place は old slot type を使う。`old_ty != new_ty` の synthetic HIR でも既存 initialized slot key と異なる new type place に誤って遷移を書かない。
- wasm / LLVM codegen では lifecycle intrinsic を runtime no-op として扱い、引数評価だけを保持する。slot safety authority は runtime helper ではなく Resource IR checker に置く。
- public stdlib wrapper は追加していない。wrapper を通常 import 可能にすると user source が lifecycle event を偽造できるため、現段階では compiler-owned intrinsic + source capability proof を producer boundary とする。

## 検証

- `cargo check -p nepl-core`
- `cargo test -p nepl-core collection_slot_lifecycle_intrinsic -- --test-threads=1`
- `cargo test -p nepl-core collection_slot_storage_dealloc_requires_owner_token_anchor -- --test-threads=1`
- `cargo test -p nepl-core collection_slot_lifecycle_boundary_uses_typed_source_evidence -- --test-threads=1`
- `cargo test -p nepl-core compiler_memory_value_type_requires_applied_element_type -- --test-threads=1`
- `cargo test -p nepl-core collection_slot_lifecycle_primitive_uses_old_type_as_replace_target -- --test-threads=1`
- `cargo test -p nepl-core resource_ir_collection_slot -- --test-threads=1`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
