---
id: ISS-20260521T141951855Z-COLLECTION-SLOT-LIFECYCLE-MODULE-EXC-964028BB
title: "Collection slot lifecycle module exceeds resource checker responsibility limit"
area: tools
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_lifecycle_*.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260521T141951855Z-COLLECTION-SLOT-LIFECYCLE-MODULE-EXC-964028BB: Collection slot lifecycle module exceeds resource checker responsibility limit

## 概要

collection_slot_lifecycle.rs grew to 221 lines after adding generic payload type checks, so nodesrc/test_resource_checker_responsibility.js fails its 200-line responsibility split limit. The lifecycle model, transition logic, and payload type matching are concentrated in one module.

## 対象

- `nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_lifecycle_*.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `collection_slot_lifecycle.rs has 221 lines; responsibility split limit is 200` で失敗した。
- collection slot proof の拡張後、lifecycle state model、transition、type-pattern matching、state table release/identity/merge tests が同じ責務境界に集まり、監査対象が読みにくくなっていた。

## 問題

collection_slot_lifecycle.rs grew to 221 lines after adding generic payload type checks, so nodesrc/test_resource_checker_responsibility.js fails its 200-line responsibility split limit. The lifecycle model, transition logic, and payload type matching are concentrated in one module.

## 影響

Resource IR static-check code can regress toward monolithic modules. If this is waived instead of split, future collection slot proof changes become harder to audit and static-check implementation mistakes become less visible.

## 修正方針

Split collection slot lifecycle into explicit model and transition modules, keep enum definitions and transition logic exhaustively matched, and update the public re-export boundary without weakening the responsibility policy.

## 対応

- `collection_slot_lifecycle` を public re-export boundary に縮小し、state/event/refutation model を `collection_slot_lifecycle_model.rs`、transition logic を `collection_slot_lifecycle_transition.rs` に分離した。
- lifecycle の regression test を generic payload type と storage dealloc の観点ごとに分け、transition 本体の責務とテスト fixture の肥大化を分離した。
- `CollectionSlotStateTable` から slot identity 判定と storage release precondition を専用 module へ移し、merge / relocate / transfer / summary replay が同じ identity helper を共有する構造にした。
- `collection_slot_state_merge` と `collection_slot_state_table` の test module も分割し、実装 module と test module の責務上限を個別に監視できるようにした。
- `nodesrc/test_resource_checker_responsibility.js` は新規分割 file をすべて監視対象に追加した。既存上限の緩和ではなく、分割後の各責務に合わせて小さい上限を与えた。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core --lib collection_slot -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js check --dir issues`: passed
