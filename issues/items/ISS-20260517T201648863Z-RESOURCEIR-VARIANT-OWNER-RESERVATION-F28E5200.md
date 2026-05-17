---
id: ISS-20260517T201648863Z-RESOURCEIR-VARIANT-OWNER-RESERVATION-F28E5200
title: "ResourceIR variant owner reservation treats Copy payloads as reserved linear owners"
area: CORE
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260517T201648863Z-RESOURCEIR-VARIANT-OWNER-RESERVATION-F28E5200: ResourceIR variant owner reservation treats Copy payloads as reserved linear owners

## 概要

PendingVariantOwnerEffects reserved-source checks kept unresolved enum payload sources reserved even when the source type is Copy. Matching an Ok branch after constructing a Result with a Copy Err payload could reject a later read of the original Copy value as use_after_move.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `PendingVariantOwnerEffects::reserved_source_for` は pending enum payload source と対象 place の overlap だけを見ており、resolved source type が Copy である場合も reserved owner source として扱っていた。
- `resource_ir_owner_variant_reservation_ignores_copy_payload_sources` で、`Result<i32, SpanLike>` の Err payload に Copy struct を渡した後、Ok arm で元の `span.start` を読む case を固定した。
- 同じ regression には non-Copy `str` の `Result<i32, str>` path も含め、Copy source skip が非Copy owner reservation 全体を無効化しないことを確認する入口を残した。

## 問題

PendingVariantOwnerEffects reserved-source checks kept unresolved enum payload sources reserved even when the source type is Copy. Matching an Ok branch after constructing a Result with a Copy Err payload could reject a later read of the original Copy value as use_after_move.

## 影響

The static checker can report memory-safety style owner errors for safe Copy payload views. This blocks self-host doctests and makes ResourceIR owner diagnostics less trustworthy.

## 修正方針

When resolving pending variant reserved sources, consult TypeCtx and skip Copy resolved source places. Keep non-Copy owner-backed sources reserved so linear owner safety remains strict.

## 検証

Add a ResourceIR regression where Result<i32, CopyStruct> permits reuse of the original Copy struct after matching Ok, while a non-Copy str path remains present for coverage. Run the focused cargo test and formatting.

## 2026-05-17 修正

`PendingVariantOwnerEffects::reserved_source_for` が `TypeCtx` を参照し、resolved source type が `Copy` の場合は pending reservation の対象から除外するようにした。これにより `Result` や他の enum variant の未解決 payload source が Copy value である場合、元 value の再読を linear owner use-after-move と誤判定しない。

非Copy source は従来通り pending reservation として扱うため、owner-bearing payload の二重使用防止は維持する。根拠は stdlib module 名や helper 名ではなく、type checker が構築した Copy capability と ResourceIR place resolution に限定した。
