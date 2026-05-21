---
id: ISS-20260521T075313201Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-7DDCFFFD
title: "Collection slot nested return transfer uses final raw aliases for call arguments"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T075313201Z-COLLECTION-SLOT-NESTED-RETURN-TRANSF-7DDCFFFD: Collection slot nested return transfer uses final raw aliases for call arguments

## 概要

Nested collection-slot return-transfer composition canonicalizes instantiated callee summary sources with the raw alias table from the end of the wrapper block. If a raw owner alias used as a call argument is rebound after the call but before the wrapper returns the call output, the summary can miss the callsite parameter relation.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- `collection_slot_summary_return_collect.rs` は callee summary の `return_transfers` を wrapper call args へ instantiate した後、wrapper block 終端の raw alias table で owner-cell canonicalization していた。
- `resource_ir_collection_slot_call_summary_uses_callsite_raw_alias_for_nested_return_transfer` は、wrapper が raw owner alias 経由で `return_err_storage(alias)` を呼び、その alias を別 storage へ rebind してから call output を返すと、修正前に caller の `LiveSlotDuringStorageDealloc` が消えることを再現した。
- call output 自体ではなく nested `Result::Err(storage)` payload を使うことで、call result の raw alias ではなく call argument の raw alias state が callsite 由来であるべきことを確認した。

## 問題

Nested collection-slot return-transfer composition canonicalizes instantiated callee summary sources with the raw alias table from the end of the wrapper block. If a raw owner alias used as a call argument is rebound after the call but before the wrapper returns the call output, the summary can miss the callsite parameter relation.

## 影響

Owner-preserving helpers that pass storage through raw owner aliases can lose caller slot state across wrapper functions, allowing storage deallocation with live non-Copy payloads to be accepted.

## 修正方針

Resolve raw owner aliases for call/indirect-call summary composition from the callsite Resource IR state, not from final block state. Reuse the generic ResourceCheckEngine state transition instead of stdlib or helper-name allowlists.

## 修正内容

- return-transfer 収集へ block entry の `CollectionSlotSummaryBuildState` を渡すようにし、producer 直前の `prior_ops` を `summary_check_engine(...).check_ops(...)` で再生して callsite state を復元する。
- direct / indirect call summary composition は、復元した callsite `raw_aliases` と `function_aliases` から source/allee を解決する。
- return value の direct alias 判定も、対象 `ops` を再生した時点の raw alias state を使う。
- raw alias の別実装や stdlib/helper-name allowlist は追加せず、既存 Resource IR state transition を再利用した。

## 検証

Add a regression where a wrapper calls identity_storage through a raw owner alias of its parameter, then rebinds that alias before returning the call output; the caller must still diagnose LiveSlotDuringStorageDealloc.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_uses_callsite_raw_alias_for_nested_return_transfer -- --nocapture`: passed
