---
id: ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31
title: "Collection slot direct return transfer ignores raw owner aliases"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31: Collection slot direct return transfer ignores raw owner aliases

## 概要

Direct return-transfer collection checks only the syntactic suffix between the returned value and function parameters. raw_aliases is available, but the direct parameter path does not canonicalize owner cell addresses before summary_place_for_params, unlike nested callee summary composition.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- 未記入

## 問題

Direct return-transfer collection checks only the syntactic suffix between the returned value and function parameters. raw_aliases is available, but the direct parameter path does not canonicalize owner cell addresses before summary_place_for_params, unlike nested callee summary composition.

## 影響

A helper can return a storage value through a raw owner alias of a parameter while the collection slot lifecycle summary fails to transfer the parameter slot state to the returned value, hiding live non-Copy payloads from callers.

## 修正方針

Canonicalize direct returned values through RawCellAddressAliases before mapping them to parameter summary places, and keep suffix/type handling consistent with nested return-transfer composition. Add a source/Resource-IR regression that returns a parameter storage through an owner alias.

## 検証

Add a Resource IR regression where a callee returns a storage parameter through an owner-cell raw alias and the caller still receives LiveSlotDuringStorageDealloc when deallocating the returned storage with a live slot.
