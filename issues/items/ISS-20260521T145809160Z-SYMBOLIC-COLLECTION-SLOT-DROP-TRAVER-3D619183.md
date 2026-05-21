---
id: ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183
title: "Symbolic collection slot drop traversal can certify storage cleanup without range proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_drop_traversal.rs, nepl-core/src/resource/collection_slot_state_release.rs, nepl-core/src/resource/collection_slot_state_identity.rs"
---

# ISS-20260521T145809160Z-SYMBOLIC-COLLECTION-SLOT-DROP-TRAVER-3D619183: Symbolic collection slot drop traversal can certify storage cleanup without range proof

## 概要

CollectionSlotDropTraversal enumerates CollectionSlotStateTable entries under storage and accepts symbolic or unknown-offset slots as if they were concrete finite slots. A single symbolic slot proof can therefore stand in for an entire dynamic initialized range without a forall/range proof.

## 対象

- `nepl-core/src/resource/collection_slot_drop_traversal.rs`
- `nepl-core/src/resource/collection_slot_state_release.rs`
- `nepl-core/src/resource/collection_slot_state_identity.rs`

## 根拠

- `CollectionSlotStateTable` は explicit slot entry を列挙できるが、`ResourceOffset::Symbolic` / `ScaledSymbolic` / `Unknown` を含む entry は「ある一つの抽象 slot」を表すだけで、`0 <= i < initialized_len` の全要素が drop 済みであることを証明しない。
- `CollectionSlotDropTraversal` が symbolic slot を通常の concrete slot と同じように受け入れると、dynamic initialized range の forall proof がないまま storage cleanup proof になり得る。
- `StorageDealloc` 側も symbolic moved / dropped slot を具体的な vacant slot と見なすと、同じ range proof 不足を release boundary で隠せる。

## 問題

CollectionSlotDropTraversal enumerates CollectionSlotStateTable entries under storage and accepts symbolic or unknown-offset slots as if they were concrete finite slots. A single symbolic slot proof can therefore stand in for an entire dynamic initialized range without a forall/range proof.

## 影響

Non-Copy Vec/OwnedBuffer cleanup could be opened using loop-shaped symbolic slot state while the compiler has only proven one arbitrary element drop, allowing storage dealloc to hide live owner payloads.

## 修正方針

Reject symbolic/unknown collection slot targets at the existing explicit-slot DropTraversal boundary and require a separate typed range proof before dynamic initialized_len cleanup can be accepted. Keep concrete explicit-slot traversal valid.

## 対応

- collection slot identity helper に `slot_requires_range_proof` を追加し、storage prefix 配下の symbolic / scaled-symbolic / unknown offset を range proof 必須の slot として分類した。
- `CollectionSlotDropTraversal` は symbolic slot を見つけた時点で `RangeProofRequired { operation: DropTraversal }` を返し、明示 slot traversal と dynamic range traversal を混同しないようにした。
- `StorageDealloc` は symbolic slot state が `Moved` / `Dropped` / `Initialized` であっても range proof なしには release を通さず、`RangeProofRequired { operation: StorageDealloc }` を返す。
- diagnostic code を `resource.collection_slot.range_proof_required` として enum に追加し、文字列判定ではなく typed refutation で扱う。
- 正の dynamic `initialized_len` range proof は別途 `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543` の残件として扱う。この issue では「証明できていない範囲を証明済み扱いしない」安全側の境界を閉じた。

## 検証

- `resource_ir_collection_slot_drop_traversal_rejects_symbolic_slot_without_range_proof` を追加し、symbolic scaled slot の drop traversal が `RangeProofRequired` で拒否され、slot state が initialized のまま残ることを固定した。
- `resource_ir_collection_storage_dealloc_rejects_moved_symbolic_slot_without_range_proof` を追加し、symbolic moved slot が range proof なしに storage dealloc を証明しないことを固定した。
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_drop_traversal -- --test-threads=1`: passed
- `cargo test -p nepl-core --lib collection_slot -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js check --dir issues`: passed
