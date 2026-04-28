---
id: ISS-20260428T130545885Z-RESOURCE-EFFECT-CHECKER-LOSES-RAW-ID-D9758A4A
title: "Resource effect checker loses raw identities read from aggregate fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T130545885Z-RESOURCE-EFFECT-CHECKER-LOSES-RAW-ID-D9758A4A: Resource effect checker loses raw identities read from aggregate fields

## 概要

ResourceOp::Construct merges raw identity into the aggregate output root, but it does not attach the identity to the corresponding field projection. A later ResourceOp::Read from the constructed field can therefore copy no identity to its output.

## 対象

- `nepl-core/src/resource/effect.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5: effect model の拡張

## 根拠

- `ResourceOp::Construct` は input raw identity を aggregate output root へ merge していたため、aggregate 全体を pure function から返すケースは検出できた。
- 一方で field projection には raw identity を付けていなかったため、`ResourceOp::Read { source: aggregate.field }` は identity を output へ copy できなかった。
- `RawIdentityTable::copy_identity` は exact place の group だけを target へ写しており、whole aggregate copy 時に descendant projection identity を target descendant へ移せなかった。

## 問題

ResourceOp::Construct merges raw identity into the aggregate output root, but it does not attach the identity to the corresponding field projection. A later ResourceOp::Read from the constructed field can therefore copy no identity to its output.

## 影響

A pure function can allocate an internal raw address, store it in a struct/tuple/enum payload, read that field back, and return the raw address without RawAddressEscapeFromInternalAlloc being reported.

## 修正方針

Represent raw identity propagation through aggregate construction at both the aggregate root and deterministic field projection, and make raw identity copies preserve descendant projection identities when whole aggregates are copied.

## 修正内容

- `ResourceOp::Construct` で input raw identity を aggregate root だけでなく deterministic field projection にも伝播するようにした。
- `RawIdentityTable::copy_identity` / `merge_identity` を prefix-aware にし、whole aggregate を copy / branch value / match value として移す場合も descendant projection の raw identity を target 側 projection へ写すようにした。
- target overwrite 時は target 配下の古い descendant identity を消してから新しい identity group を構築し、stale field identity を残さないようにした。

## 検証

- `resource_ir_effect_check_reports_raw_alloc_escape_read_from_constructed_aggregate_field` を追加した。
- `resource_ir_effect_check_preserves_raw_identity_fields_across_aggregate_copy` を追加した。
- 修正前は constructed aggregate field read の targeted regression が失敗することを確認した。
- 修正後に effect 系 focused regression は成功済み。最終確認として `resource_ir` 全体、`trunk build`、issue check、rustfmt check、diff check を実行する。
