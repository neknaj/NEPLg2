---
id: ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31
title: "Collection slot direct return transfer ignores raw owner aliases"
area: core
status: fixed
resolved: true
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

- `collection_slot_summary_return_collect.rs` の direct return-transfer path は、返却 `Place` と parameter `Place` の syntactic prefix だけを比較していた。
- 同じ file の nested callee summary composition は `raw_aliases.canonicalize_owner_cell_address` 後に `summary_place_for_params` へ渡しており、direct path と nested path で raw owner alias の扱いが非対称だった。
- `resource_ir_collection_slot_call_summary_transfers_caller_slot_through_returned_raw_owner_alias` は、callee が parameter storage と同じ owner cell の raw alias place を返すと、修正前に caller 側の live slot dealloc 診断が消えることを再現した。

## 問題

Direct return-transfer collection checks only the syntactic suffix between the returned value and function parameters. raw_aliases is available, but the direct parameter path does not canonicalize owner cell addresses before summary_place_for_params, unlike nested callee summary composition.

## 影響

A helper can return a storage value through a raw owner alias of a parameter while the collection slot lifecycle summary fails to transfer the parameter slot state to the returned value, hiding live non-Copy payloads from callers.

## 修正方針

Canonicalize direct returned values through RawCellAddressAliases before mapping them to parameter summary places, and keep suffix/type handling consistent with nested return-transfer composition. Add a source/Resource-IR regression that returns a parameter storage through an owner alias.

## 修正内容

- direct return-transfer 収集で返却 value を `RawCellAddressAliases::canonicalize_owner_cell_address` に通してから `summary_place_for_params` へ渡すようにした。
- これにより syntactic prefix だけでなく、Resource IR 上で証明済みの owner-cell raw alias を parameter-relative source として扱う。
- stdlib module 名、関数名、`Result` 名などの allowlist は追加していない。

## 検証

Add a Resource IR regression where a callee returns a storage parameter through an owner-cell raw alias and the caller still receives LiveSlotDuringStorageDealloc when deallocating the returned storage with a live slot.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_transfers_caller_slot_through_returned_raw_owner_alias -- --nocapture`: passed
