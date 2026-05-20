---
id: ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C
title: "Collection slot lifecycle needs place-indexed proof state"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/collection_slot_state_table.rs, nepl-core/src/resource/mod.rs"
---

# ISS-20260520T184420541Z-COLLECTION-SLOT-LIFECYCLE-NEEDS-PLAC-2B2D025C: Collection slot lifecycle needs place-indexed proof state

## 概要

The typed collection slot lifecycle transition exists only as a single-state function, so lowering/checker code still lacks a generic place-indexed proof state for slots under collection storage.

## 対象

- `nepl-core/src/resource/collection_slot_lifecycle.rs, nepl-core/src/resource/mod.rs`

## 根拠

- [ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D](./ISS-20260520T183033547Z-NON-COPY-COLLECTION-PAYLOADS-NEED-TY-674BA21D.md) で単一 slot の typed transition は追加済みだが、Resource IR が複数 slot を扱うには `Place` ごとの state table が必要だった。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、stdlib module allowlist ではなく compiler-core の汎用 proof boundary で collection payload state を扱う方針を示している。
- raw `mem_copy` / `mem_move` は Copy-only byte operation のまま保つため、non-Copy payload の move/drop は raw memory state ではなく typed collection slot state として別に証明する必要がある。

## 問題

The typed collection slot lifecycle transition exists only as a single-state function, so lowering/checker code still lacks a generic place-indexed proof state for slots under collection storage.

## 影響

Resource IR cannot connect non-Copy collection payload operations to one reusable proof engine without rebuilding ad-hoc module-specific state, which would violate the generic static-check policy.

## 修正方針

Add a compiler-core collection slot table keyed by Resource IR Place and route all slot lifecycle events through the same enum transition boundary.

## 対応

- `CollectionSlotStateTable` を追加し、slot `Place` ごとの `CollectionSlotState` を保持するようにした。
- `apply_slot_event` は `CollectionSlotLifecycleEvent` をそのまま受け取り、既存の enum transition boundary へ委譲する。
- `release_storage` は storage 配下の live initialized slot を拒否し、moved/dropped slot だけを release できるようにした。
- `CollectionSlotTableRefutation` は slot と typed refutation を保持し、将来の diagnostics が generic checker error に潰れないようにした。
- `resource` module から table/entry/refutation を re-export し、lowering/checker が同じ proof state を使える境界にした。

## 検証

cargo test -p nepl-core collection_slot_lifecycle -- --test-threads=1
