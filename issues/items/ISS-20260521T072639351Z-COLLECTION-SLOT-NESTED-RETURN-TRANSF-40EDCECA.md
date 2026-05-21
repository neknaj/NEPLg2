---
id: ISS-20260521T072639351Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-40EDCECA
title: "Collection slot nested return transfer needs function-alias indirect regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260521T072639351Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-40EDCECA: Collection slot nested return transfer needs function-alias indirect regression

## 概要

The nested collection slot return transfer fix composes callee return_transfers for both direct and function-alias indirect calls, but only the direct wrapper path is covered. Without an indirect-call regression, future changes can break function alias summary composition while direct call tests stay green.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- [ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78](./ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78.md) で direct call / function-alias indirect call の両方を generic summary composition に載せた。
- direct call path は `resource_ir_collection_slot_call_summary_transfers_caller_slot_through_nested_returned_enum_payload` で固定済みだが、function value / indirect call path の回帰は未固定だった。
- Stage 6 の方針では、higher-order helper composition も stdlib/module allowlist ではなく Resource IR の function alias summary と place suffix で証明する必要がある。

## 問題

The nested collection slot return transfer fix composes callee return_transfers for both direct and function-alias indirect calls, but only the direct wrapper path is covered. Without an indirect-call regression, future changes can break function alias summary composition while direct call tests stay green.

## 影響

Owner-preserving self-host abstractions that pass helper functions through values could lose non-Copy collection slot state across fallible wrappers, weakening memory-safety checks for higher-order helper composition.

## 修正方針

Add a Resource IR regression where a wrapper stores identity_storage as a FunctionValue, calls it through IndirectCall, wraps the returned storage in StorageResult::Err, and the caller match bind detects LiveSlotDuringStorageDealloc on recovered storage.

## 対応

- nested return transfer regression fixture を helper 化し、direct call と function-alias indirect call の両方を同じ構造で検査できるようにした。
- indirect variant では wrapper が `ResourceOp::FunctionValue` で `identity_storage` を値として保持し、`ResourceOp::IndirectCall` の output を `StorageResult::Err` payload に包んで返す。
- caller は direct variant と同じく `Err recovered` を match bind し、`StorageDealloc` が `LiveSlotDuringStorageDealloc` を報告することを確認する。
- これにより `FunctionAliasTable` から callee summary を読む経路も回帰で固定し、direct call だけが green のまま indirect summary composition が壊れる退行を検出できる。

## 検証

Run the new focused resource_ir test, resource_ir_collection_slot_call_summary filter, cargo check -p nepl-core, cargo fmt --check, node nodesrc/issues.js check --dir issues, and git diff --check.

実施:

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_transfers_caller_slot_through_indirect_nested_returned_enum_payload -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
