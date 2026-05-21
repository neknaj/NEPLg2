---
id: ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347
title: "Collection slot summaries do not transfer caller slot state through returned owner parameters"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_*.rs"
---

# ISS-20260521T004808159Z-COLLECTION-SLOT-SUMMARIES-DO-NOT-TRA-2AA84347: Collection slot summaries do not transfer caller slot state through returned owner parameters

## 概要

A function that returns an owner parameter without creating new collection slot events has no collection-slot summary facts. At the caller, check_direct_call consumes the argument but apply_call_collection_slot_lifecycle_summary clears only the output prefix, so live slot state can remain under the moved argument and fail to follow the returned owner. Later storage dealloc of the returned owner can miss initialized non-Copy payload state.

## 対象

- `nepl-core/src/resource/collection_slot_summary_*.rs`

## 根拠

- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の残件である owner-preserving collection update / move-out / drop traversal は、callee を跨いでも caller 側の slot state が同じ storage identity に追従する必要がある。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、stdlib module ごとの個別許可ではなく Resource IR の generic proof boundary に initialized / moved / dropped state を載せる方針である。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、`OwnedBuffer<T>` の initialized slot state と owner-preserving API を self-host 前の memory-safety 基盤として扱う。

## 問題

A function that returns an owner parameter without creating new collection slot events has no collection-slot summary facts. At the caller, check_direct_call consumes the argument but apply_call_collection_slot_lifecycle_summary clears only the output prefix, so live slot state can remain under the moved argument and fail to follow the returned owner. Later storage dealloc of the returned owner can miss initialized non-Copy payload state.

## 影響

Owner-preserving collection APIs can hide live non-Copy payload slots across function calls, undermining the generic Resource IR proof for initialized/moved/dropped collection storage.

## 修正方針

Represent return-time collection slot value transfer in the generic collection slot function summary, instantiate it at call sites, and transfer the caller actual argument prefix to the call output before applying explicit returned slot facts.

## 対応

- `CollectionSlotLifecycleFunctionSummary` に `return_transfers` を追加し、callee の return value が parameter prefix そのものを返す場合に、parameter-relative source と output-relative target を typed summary fact として保持するようにした。
- direct / indirect call summary application は、callee 内 lifecycle ops を replay した後、call output の stale slot state を消し、return transfer を caller actual argument から call output へ適用してから explicit return slot state を設定する。
- transfer source / target は既存の raw alias canonicalization を通す。これにより、owner value place と underlying raw storage identity のどちらか片方にだけ slot state が残る状態を避ける。
- stdlib 関数名や module 名の allowlist は追加していない。return value と parameter prefix の構文・Resource IR place 関係から generic summary fact を導く。

## 検証

Add Resource IR regression for identity owner return with a live slot, plus focused collection slot summary tests, cargo check/fmt, and issues check.

- `cargo test -p nepl-core resource_ir_collection_slot_call_summary_transfers_caller_slot_through_returned_parameter --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core resource_ir_collection_slot_call_summary_ --test resource_ir -- --test-threads=1`
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`
- `cargo fmt --check -p nepl-core`
- `cargo check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
