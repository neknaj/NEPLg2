---
id: ISS-20260507T155948337Z-SELFHOST-MONO-INSTANCE-IDS-USE-1-INV-434774DA
title: "Selfhost mono instance IDs use -1 invalid sentinel"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/mono/mono.nepl, nodesrc/test_selfhost_mono_instance_absence.js"
---

# ISS-20260507T155948337Z-SELFHOST-MONO-INSTANCE-IDS-USE-1-INV-434774DA: Selfhost mono instance IDs use -1 invalid sentinel

## 概要

SelfhostMonoInstanceId exposes selfhost_mono_instance_id_invalid and selfhost_mono_instance_id_is_valid, representing an unassigned instance as index -1 instead of typed absence.

## 対象

- `stdlib/neplg2/core/mono/mono.nepl`
- `nodesrc/test_selfhost_mono_instance_absence.js`

## 根拠

- `SelfhostMonoInstanceId` が stable table index である一方、未割り当て状態を `selfhost_mono_instance_id_invalid` で `index = -1` として表していた。
- `selfhost_mono_instance_id_is_valid` が存在し、ID 利用側が値検査で未割り当てを弾く設計になっていた。
- monomorphize cache lookup の「未登録」と「登録済み ID」は ID 自体ではなく `Option<SelfhostMonoInstanceId>` の payload 有無で表すべきである。

## 問題

SelfhostMonoInstanceId exposes selfhost_mono_instance_id_invalid and selfhost_mono_instance_id_is_valid, representing an unassigned instance as index -1 instead of typed absence.

## 影響

Monomorphize cache and codegen work can accidentally pass an unassigned instance ID as an ordinary table index. This violates the self-host typed model policy that absence must be represented by Option or enum payloads instead of numeric sentinels.

## 修正方針

Remove invalid instance ID construction and represent pending/unassigned instance lookup state as Option<SelfhostMonoInstanceId>. Keep SelfhostMonoInstanceId as a stable table index only.

## 検証

Add a source policy rejecting selfhost_mono_instance_id_invalid, instance_id_is_valid, and mono instance -1 construction. Run focused mono doctest, issue check, and source policy regressions.

## 対応結果

- `selfhost_mono_instance_id_invalid` を削除した。
- `selfhost_mono_instance_id_is_valid` を削除し、`SelfhostMonoInstanceId` を stable table index の値に限定した。
- `selfhost_mono_instance_id_pending` を追加し、未割り当てを `Option<SelfhostMonoInstanceId>::None` として返すようにした。
- `selfhost_mono_instance_id_assigned` を追加し、割り当て済み ID を `Some` payload として返すようにした。
- `nodesrc/test_selfhost_mono_instance_absence.js` を追加し、invalid constructor、validity helper、`-1` instance construction の再導入を source policy で拒否する。

## 検証結果

- `node nodesrc/test_selfhost_mono_instance_absence.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/mono/mono.nepl --no-tree -o tmp/agent1-selfhost-mono-instance-absence.json -j 1 --dist web/dist`: total=1, passed=1
