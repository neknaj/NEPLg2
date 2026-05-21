---
id: ISS-20260521T080236863Z-COLLECTION-SLOT-RETURN-TRANSFER-IGNO-1F875A73
title: "Collection slot return transfer ignores match bind payload state"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T080236863Z-COLLECTION-SLOT-RETURN-TRANSFER-IGNO-1F875A73: Collection slot return transfer ignores match bind payload state

## 概要

Return-transfer collection descends into ResourceOp::Match arms without applying the arm bind payload relation to the per-arm summary state. A callee that returns a storage owner extracted from an enum payload can fail to summarize the transfer from the scrutinee parameter payload.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- `collection_slot_summary_return_collect.rs` は `ResourceOp::Match` の arm を再帰的に辿る際、scrutinee 評価後の state は使っていたが、arm bind local が scrutinee payload から作られた owner であることを raw alias / collection slot state へ反映していなかった。
- `resource_ir_collection_slot_call_summary_transfers_match_bound_returned_payload` は、callee が `StorageResult::Err(storage)` を match して bound storage を返すと、修正前に caller 側の `LiveSlotDuringStorageDealloc` が消えることを再現した。
- `Result` や特定 stdlib module の allowlist ではなく、`ResourceMatchArm` と `match_bind_payload_place` から得られる Resource IR の payload relation だけで transfer を証明できることを確認した。

## 問題

Return-transfer collection descends into ResourceOp::Match arms without applying the arm bind payload relation to the per-arm summary state. A callee that returns a storage owner extracted from an enum payload can fail to summarize the transfer from the scrutinee parameter payload.

## 影響

Owner-preserving APIs that destructure Result or collection-specific error enums before returning storage can lose live non-Copy slot state, weakening memory safety for fallible collection operations and self-host compiler data structures.

## 修正方針

Build match-arm entry state from the match scrutinee payload: mark bind locals initialized, copy raw/function aliases from the payload, and transfer collection slot state to the bind local before collecting arm return transfers. Keep the proof generic over Resource IR match payloads.

## 修正内容

- return-transfer 収集で `ResourceOp::Match` arm へ入る直前に、match scrutinee 評価後の `CollectionSlotSummaryBuildState` から arm entry state を作る helper を追加した。
- arm bind local は initialized として扱い、`match_bind_payload_place` が返す payload source から raw owner alias と collection slot state を bind local へ伝播する。
- function value payload についても既存の `function_aliases_for_match_arm` を同じ arm entry state に統合し、match payload の value / function / slot relation を同じ Resource IR state として扱う。
- 特定 enum 名、variant 名、stdlib function 名の allowlist は追加していない。

## 検証

Add a regression where a helper matches Result::Err(storage), returns the bound storage, and the caller deallocates the returned storage with a live slot; the diagnostic must remain LiveSlotDuringStorageDealloc.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_transfers_match_bound_returned_payload -- --nocapture`: passed
