---
id: ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521
title: "Collection slot state does not follow owner value transfers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/**"
---

# ISS-20260520T223249968Z-COLLECTION-SLOT-STATE-DOES-NOT-FOLLO-A808C521: Collection slot state does not follow owner value transfers

## 概要

CollectionSlotStateTable tracks initialized/moved/dropped slot state by storage Place, but ResourceOp value-flow operations currently move raw aliases and raw cells without rekeying collection slot state. Moving an owner token into a local, aggregate field, branch/match output, or return/call output can leave slot facts under the stale source prefix, which would let later storage dealloc on the new owner miss live non-Copy payload slots.

## 対象

- `nepl-core/src/resource/**`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection payload を stdlib 個別 proof ではなく compiler-core の generic Resource IR proof boundary に載せる方針を定めている。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、`OwnedBuffer<T>` / initialized slot state / drop traversal を self-host 前の P1 基盤として扱っている。
- [ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2](./ISS-20260520T214013832Z-COLLECTION-SLOT-LIFECYCLE-STATE-DOES-FA4DE5B2.md) で storage relocate は追加済みだったが、通常の Resource IR value transfer は別経路として残っていた。

## 問題

CollectionSlotStateTable tracks initialized/moved/dropped slot state by storage Place, but ResourceOp value-flow operations currently move raw aliases and raw cells without rekeying collection slot state. Moving an owner token into a local, aggregate field, branch/match output, or return/call output can leave slot facts under the stale source prefix, which would let later storage dealloc on the new owner miss live non-Copy payload slots.

## 影響

Non-Copy Vec/OwnedBuffer support would need ad-hoc Vec-specific proof or would accept storage-only dealloc of live payloads after ordinary value movement. This violates the generic Resource IR proof boundary and can hide memory-safety bugs from the static checker.

## 修正方針

Add a generic collection-slot state transfer operation keyed by Place prefix replacement, invoke it from initialized/resource value-flow paths that consume and materialize values, and extend collection slot call summaries to carry return-relative slot state instead of relying on stdlib allowlists.

## 検証

Add Resource IR regressions for Move/Construct/Branch/Match/Call return transfer of live collection slot state and for stale source prefix not being usable after transfer.

## 2026-05-21 修正

- `CollectionSlotStateTable::transfer_storage_prefix` を追加し、source prefix 配下の initialized / moved / dropped slot state と release marker を target prefix へ rekey するようにした。
- transfer target 側に既存 slot / released marker がある場合は `CollectionSlotLifecycleOp::ValueTransfer` の typed refutation として報告し、暗黙上書きで live slot を隠さない。
- Resource IR initialized checker の `DeclareLocal` / `Read` / `Assign` / `Move` / `Construct` / branch value / match bind / match value に接続し、non-Copy owner value の通常移動で slot state が stale source prefix に残らないようにした。
- collection slot summary に `return_slots` を追加し、callee が live slot state を持つ owner を return した場合に caller の call output へ state を復元するようにした。summary 更新判定も return-only summary を空扱いしない。
- 回帰テストとして、owner value の `Move`、aggregate `Construct` field、call return summary で live slot state が移動先 storage dealloc により拒否されることを `nepl-core/tests/resource_ir.rs` に固定した。
- released storage marker も通常 value transfer で target prefix へ移ることを `collection_slot_state_transfer_tests.rs` に固定し、release marker だけが source 消去時に失われる退行を防いだ。
- call return summary は output prefix を callee return state で置換するようにし、return slot が空または release marker の場合に stale output state が残らないようにした。released / maybe released marker も return summary に含める。
- subagent review で指摘された return marker / stale output の P1 抜けを修正し、released storage return と stale output 置換の回帰テストを追加した。

検証:

- `cargo test -p nepl-core collection_slot -- --test-threads=1`
- `cargo check -p nepl-core`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
