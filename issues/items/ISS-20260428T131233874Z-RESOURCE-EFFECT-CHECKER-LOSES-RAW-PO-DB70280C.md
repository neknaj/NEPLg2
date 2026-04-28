---
id: ISS-20260428T131233874Z-RESOURCE-EFFECT-CHECKER-LOSES-RAW-PO-DB70280C
title: "Resource effect checker loses raw pointer aliases stored in aggregate fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T131233874Z-RESOURCE-EFFECT-CHECKER-LOSES-RAW-PO-DB70280C: Resource effect checker loses raw pointer aliases stored in aggregate fields

## 概要

ResourceOp::Construct does not propagate raw pointer alias state from an input pointer to the corresponding aggregate field projection. Reads from that field therefore produce a pointer value that is no longer aliased to the original raw slot.

## 対象

- `nepl-core/src/resource/effect.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5: effect model の拡張

## 根拠

- `RawPointerAliasTable` は raw slot payload identity を pointer alias group 単位で key 化するため、slot pointer の copy / projection を正確に保持する必要がある。
- `ResourceOp::Construct` は input pointer alias を aggregate field projection へ伝播していなかった。
- `RawPointerAliasTable::copy_alias` / `remove_place` は exact place だけを扱っており、whole aggregate copy 時に descendant field pointer alias を target field へ写せなかった。

## 問題

ResourceOp::Construct does not propagate raw pointer alias state from an input pointer to the corresponding aggregate field projection. Reads from that field therefore produce a pointer value that is no longer aliased to the original raw slot.

## 影響

A raw identity stored through a pointer read from an aggregate field can be keyed to the field pointer instead of the original slot. Later loads from the original slot can miss RawAddressEscapeFromInternalAlloc.

## 修正方針

Propagate raw pointer aliases during aggregate construction using deterministic field projections, and make pointer alias copy/remove prefix-aware so whole aggregate copies preserve descendant pointer aliases without stale target aliases.

## 修正内容

- `ResourceOp::Construct` で input pointer alias を deterministic aggregate field projection へ伝播するようにした。
- `RawPointerAliasTable::copy_alias` を prefix-aware にし、whole aggregate copy で descendant pointer alias を target descendant へ写すようにした。
- `RawPointerAliasTable::remove_place` と `RawMemoryIdentityTable::remove_place` を prefix-aware にし、target overwrite 時に stale descendant pointer / raw slot identity が残らないようにした。

## 検証

- `resource_ir_effect_check_preserves_raw_slot_pointer_alias_stored_in_aggregate_field` を追加した。
- `resource_ir_effect_check_preserves_raw_slot_pointer_alias_fields_across_aggregate_copy` を追加した。
- 修正前は aggregate field に保存した slot pointer 経由の store/load regression が失敗することを確認した。
- 修正後に pointer alias focused regression は成功済み。最終確認として `resource_ir` 全体、`trunk build`、issue check、rustfmt check、diff check を実行する。
