---
id: ISS-20260521T073615983Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-0A85EC1F
title: "Collection slot nested return transfer uses final indirect callee aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T073615983Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-0A85EC1F: Collection slot nested return transfer uses final indirect callee aliases

## 概要

IndirectCall return-transfer collection reads the FunctionAliasTable supplied after summary building has walked the whole block. If a function-valued local is reassigned after the indirect call but before the returned aggregate is constructed or returned, the collector can compose the callee summary from the later alias instead of the alias proven at the call site.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- `collection_slot_summary_build.rs` は block 内の `ops` をすべて処理した後の `FunctionAliasTable` を return-transfer 収集へ渡していた。
- `collection_slot_summary_return_collect.rs` は `ResourceOp::IndirectCall` の output producer を見つけた時点で、その final alias table から callee summary を解決していた。
- `resource_ir_collection_slot_call_summary_uses_callsite_indirect_alias_for_nested_transfer` は、indirect call 後に同じ function value place を別関数へ上書きすると、修正前に `LiveSlotDuringStorageDealloc` が消えることを再現した。

## 問題

IndirectCall return-transfer collection reads the FunctionAliasTable supplied after summary building has walked the whole block. If a function-valued local is reassigned after the indirect call but before the returned aggregate is constructed or returned, the collector can compose the callee summary from the later alias instead of the alias proven at the call site.

## 影響

The Resource IR can miss a live non-Copy collection slot transferred through an indirect callee return value, allowing a later StorageDealloc to be accepted even though the payload slot remains live.

## 修正方針

Resolve indirect callee summaries from the alias state at the producer call site, not from the final block alias table. The fix must remain source/Resource-IR based and must not rely on stdlib or function-name allowlists.

## 修正内容

- `FunctionAliasTable` に `ResourceOp` 列を enum `match` で再生する helper を追加した。
- collection slot return-transfer 収集は、block entry の function alias state から producer 直前の `prior_ops` を再生して callsite alias を復元する。
- branch / match 内の nested producer では、分岐入口や match arm bind の alias state を構成してから arm 内の producer を追跡する。
- direct / indirect の callee 名、stdlib module 名、`Result` 名の allowlist は追加していない。

## 追加で発見した問題

- [ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31](./ISS-20260521T074121324Z-COLLECTION-SLOT-DIRECT-RETURN-TRANSF-C8D61A31.md): direct return-transfer の parameter 判定が raw owner alias canonicalization を使っていない疑いがあるため、別 issue として分離した。

## 検証

Add a regression where a wrapper calls identity_storage indirectly, then overwrites the callee function variable before returning Result::Err(forwarded). The caller must still diagnose LiveSlotDuringStorageDealloc for the recovered storage payload.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_uses_callsite_indirect_alias_for_nested_transfer -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
