---
id: ISS-20260521T155723411Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-A919B9B0
title: "Symbolic collection slot drop traversal lacks generic initialized count range proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_drop_traversal_range.rs, nepl-core/src/resource/collection_slot_drop_traversal_known_range.rs, nepl-core/src/resource/collection_slot_drop_traversal_symbolic_range.rs"
---

# ISS-20260521T155723411Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-A919B9B0: Symbolic collection slot drop traversal lacks generic initialized count range proof

## 概要

CollectionSlotDropTraversal now carries initialized_count and checks known offsets, but symbolic or scaled-symbolic slot entries are still rejected even when Resource IR has source-derived nonnegative and upper-bound scalar facts. This blocks positive dynamic initialized_len cleanup for non-Copy collections and leaves the parent issue dependent on ad hoc alternatives.

## 対象

- `nepl-core/src/resource/collection_slot_drop_traversal_range.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、collection slot cleanup を stdlib module allowlist ではなく Resource IR の typed fact / obligation / evidence として扱う方針を定めている。
- [ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A](./ISS-20260521T151802959Z-COLLECTION-SLOT-DROP-TRAVERSAL-LACKS-B557D89A.md) で `initialized_count` operand は Resource IR へ通ったが、symbolic / scaled-symbolic offset は positive proof がなく安全側で拒否されたままだった。

## 問題

CollectionSlotDropTraversal now carries initialized_count and checks known offsets, but symbolic or scaled-symbolic slot entries are still rejected even when Resource IR has source-derived nonnegative and upper-bound scalar facts. This blocks positive dynamic initialized_len cleanup for non-Copy collections and leaves the parent issue dependent on ad hoc alternatives.

## 影響

Self-host collection cleanup cannot prove dynamic non-Copy slot traversal through the generic Resource IR path. Implementations may be pushed toward stdlib helper allowlists or storage-only cleanup shortcuts, both of which violate the static-check design policy.

## 修正方針

Extend the collection slot traversal range proof to reuse typed scalar relation facts for symbolic and scaled-symbolic offsets. Accept only proofs that establish the index is nonnegative, maps exactly to the element stride with no unsupported known offset, and is strictly below initialized_count. Keep unknown or insufficiently bounded offsets rejected.

## 検証

scaled symbolic slot traversal が `NonNegative` と `index < initialized_count` の fact を持つときだけ通る regression を追加する。各 fact が欠落した場合は `RangeProofRequired` のまま拒否する。

## 対応

- `CollectionSlotDropTraversal` の range proof を known offset と symbolic offset の責務に分割し、symbolic / scaled-symbolic offset は次のすべてを満たす場合だけ通すようにした。
  - `known == 0`
  - `scale == element_stride`
  - `index` が `NonNegative`
  - `index < initialized_count`
- `Unknown`、stride 不一致、zero-sized payload の symbolic offset、`NonNegative` または upper-bound fact 欠落は引き続き `RangeProofRequired` とする。
- symbolic slot が typed range proof と actual loaded-value drop proof の両方で drop traversal を通過した場合、その symbolic slot entry を `Uninitialized` として消し、後続の storage release が「証明済みで drop 済みの symbolic entry」を再度 range proof 不足として拒否しないようにした。
- `collection_slot_drop_traversal_range.rs` が肥大化しないよう、known range と symbolic range の proof helper を別 module に分割し、責務チェックに追加した。

## 回帰テスト

- `resource_ir_collection_slot_drop_traversal_accepts_symbolic_slot_with_range_proof`
- `resource_ir_collection_slot_drop_traversal_rejects_symbolic_slot_without_nonnegative_proof`
- `resource_ir_collection_slot_drop_traversal_rejects_symbolic_slot_without_upper_bound_proof`

## 検証結果

- `cargo fmt --check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_drop_traversal -- --test-threads=1`
- `node nodesrc/test_resource_checker_responsibility.js`
- `cargo check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
