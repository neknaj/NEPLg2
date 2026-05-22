---
id: ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690
title: "Collection slot release alias module exceeds responsibility limit"
area: core
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_state_release_alias.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690: Collection slot release alias module exceeds responsibility limit

## 概要

After registering the new summary projection module, the resource responsibility monitor reaches collection_slot_state_release_alias.rs and fails because the module has 130 lines while its split limit is 120.

## 対象

- `nepl-core/src/resource/collection_slot_state_release_alias.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After registering the new summary projection module, the resource responsibility monitor reaches collection_slot_state_release_alias.rs and fails because the module has 130 lines while its split limit is 120.

## 影響

The resource checker responsibility gate cannot pass on current main, so future static-check refactors lose an automated signal for release/dealloc proof module growth.

## 修正方針

Split collection_slot_state_release_alias.rs by release-state query versus release-state mutation, or otherwise reduce the module below its current responsibility budget without raising the limit.

## 対応内容

`collection_slot_state_release_alias.rs` は alias-aware storage release の mutation entrypoint に絞り、release precondition と「既に collection slot state が関係するか」の判定を `collection_slot_state_release_alias_precondition.rs` へ分離した。`CollectionSlotStateTable` の field は `pub(super)` のまま同一親 module 内で参照し、release mutation と proof/query の境界を明確にした。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and issue validation.

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core collection_slot_state_release --lib -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_storage_dealloc -- --test-threads=1 --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_raw_dealloc -- --test-threads=1 --nocapture`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: `collection_slot_state_release_alias.rs` blocker は解消。次の既存 blocker として `initialized_availability.rs has 173 lines; responsibility split limit is 120` を検出したため、`ISS-20260522T025248689Z-INITIALIZED-AVAILABILITY-MODULE-EXCE-CF822B46` に分離した。
