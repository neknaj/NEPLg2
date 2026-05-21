---
id: ISS-20260521T131208972Z-COLLECTION-SLOT-DROP-TRAVERSAL-DOES--96E42080
title: "Collection slot drop traversal does not check owner token element type"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/effects.rs"
---

# ISS-20260521T131208972Z-COLLECTION-SLOT-DROP-TRAVERSAL-DOES--96E42080: Collection slot drop traversal does not check owner token element type

## 概要

collection_slot_drop_traversal<T> requires an owner token storage anchor, but typecheck does not verify that the intrinsic type argument T matches RegionToken<T>'s element type. A mismatched source intrinsic can reach Resource IR with an inconsistent expected_ty instead of failing at the compiler-owned source boundary.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs, nepl-core/tests/effects.rs`

## 根拠

- [ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B](./ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B.md) で、collection-wide cleanup を `CollectionSlotDropTraversal` として Resource IR に載せた。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、compiler-owned source boundary と Resource IR proof boundary の両方を typed enum / match で検査する方針を定めている。
- 実装確認で、slot offset を持つ lifecycle primitive は `MemPtr<T>` / `RegionToken<T>` の element type と intrinsic type args を照合していた一方、offset を持たない `DropTraversal` は owner token anchor だけを検査し、`<T>` と `RegionToken<T>` の一致を検査していなかった。

## 問題

collection_slot_drop_traversal<T> requires an owner token storage anchor, but typecheck does not verify that the intrinsic type argument T matches RegionToken<T>'s element type. A mismatched source intrinsic can reach Resource IR with an inconsistent expected_ty instead of failing at the compiler-owned source boundary.

## 影響

Non-Copy collection cleanup proof could carry a traversal type unrelated to the storage owner token. Even if later Resource IR state rejects many concrete uses, the compiler-owned source boundary is not statically self-checking and can hide lowering/proof bugs.

## 修正方針

Treat DropTraversal as an owner-token lifecycle primitive that also has a slot target type. Validate the owner token element type against all intrinsic type arguments at typecheck, and keep StorageDealloc without type args. Add regression tests for matching and mismatched RegionToken element types.

## 対応内容

- `validate_collection_slot_lifecycle_intrinsic` で、`primitive.has_slot_offset()` だけでなく `primitive.requires_storage_drop_traversal()` の場合も `validate_collection_slot_lifecycle_anchor_value_type` を呼ぶようにした。
- `DropTraversal` は storage-only operation なので owner token anchor は引き続き要求するが、`StorageDealloc` と異なり type arg を持つため、`RegionToken<T>` の element type と intrinsic `<T>` を typecheck 境界で照合する。
- 回帰テストとして、`collection_slot_drop_traversal<i32>(RegionToken<i32>)` は通し、`collection_slot_drop_traversal<u8>(RegionToken<i32>)` は `IntrinsicArgTypeMismatch` で拒否する case を追加した。

## 検証

cargo test -p nepl-core --test effects collection_slot_drop_traversal -- --test-threads=1
