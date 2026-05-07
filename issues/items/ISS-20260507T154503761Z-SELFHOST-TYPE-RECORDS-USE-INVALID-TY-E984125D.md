---
id: ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D
title: "Selfhost type records use invalid TypeId and numeric primitive ranges"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/ty/ty.nepl, nodesrc/test_selfhost_type_record_payload.js"
---

# ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D: Selfhost type records use invalid TypeId and numeric primitive ranges

## 概要

SelfhostTypeRecord stores kind, first_arg, arg_count, and result for every type. Primitive records fill first_arg with -1 and result with selfhost_type_id_invalid even though those fields are meaningful only for functions.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl`
- `nodesrc/test_selfhost_type_record_payload.js`

## 根拠

- `SelfhostTypeRecord` が primitive/function の区別に関係なく `kind` / `first_arg` / `arg_count` / `result` を持っていた。
- primitive record は `first_arg = -1` と `selfhost_type_id_invalid` を埋めることで function-only payload を無効化しており、record 単体で invalid state を表現できた。
- function type かどうかを `kind` と payload field の組み合わせで判断するため、accessor や比較処理で field 読み間違いを静的に防げない構造だった。

## 問題

SelfhostTypeRecord stores kind, first_arg, arg_count, and result for every type. Primitive records fill first_arg with -1 and result with selfhost_type_id_invalid even though those fields are meaningful only for functions.

## 影響

Type layer users can accidentally read function-only payload from primitive records, and the self-host model keeps invalid TypeId state in normal records. This weakens the enum-first static-check design required before self-host type/effect checking.

## 修正方針

Split SelfhostTypeRecord into primitive and function payload variants. Primitive records must carry only SelfhostTypeKind, function records must carry argument range and result TypeId, and all accessors/comparisons must match the enum payload.

## 検証

Add a source policy rejecting selfhost_type_id_invalid, primitive first_arg = -1 records, and direct record field reads that bypass SelfhostTypeRecord payload matching. Run focused ty doctests, issue check, and source policy regressions.

## 対応結果

- `SelfhostTypeRecord` を flat struct から `Primitive <SelfhostPrimitiveTypeKind>` / `Function <SelfhostFunctionTypeRecord>` の enum payload に変更した。
- `SelfhostPrimitiveTypeKind` を追加し、primitive payload から `Function` variant を除外した。
- `SelfhostFunctionTypeRecord` に `first_arg` / `arg_count` / `result` を集約し、function type だけが function-only field を持つ設計にした。
- `selfhost_type_id_invalid` と flat `selfhost_type_record_new` を削除した。
- `selfhost_type_arena_get_kind` / `function_arg_count` / `function_arg` / `function_result` / `records_equal` は `SelfhostTypeRecord` を直接 `match` する形にした。
- `nodesrc/test_selfhost_type_record_payload.js` を追加し、flat record、invalid TypeId helper、primitive `-1` range、payload match bypass の再導入を source policy で拒否する。

## 検証結果

- `node nodesrc/test_selfhost_type_record_payload.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/ty/ty.nepl --no-tree -o tmp/agent1-selfhost-type-record-payload.json -j 1 --dist web/dist`: total=1, passed=1
