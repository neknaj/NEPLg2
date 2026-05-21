---
id: ISS-20260521T081809182Z-COLLECTION-SLOT-RETURN-TRANSFER-MATC-F51ED8E2
title: "Collection slot return-transfer match entry state is incomplete"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T081809182Z-COLLECTION-SLOT-RETURN-TRANSFER-MATC-F51ED8E2: Collection slot return-transfer match entry state is incomplete

## 概要

Return-transfer collection constructs match arm state with only raw/function alias and collection slot transfer, while the generic Resource IR match semantics also handle arm reachability, pending variant initialization, pending realloc result copies, and loaded value origin transfer.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- return-transfer 収集の `collection_slot_summary_match_arm_entry_state` は、match arm bind local の initialized / raw alias / function alias / collection slot transfer だけを扱っていた。
- 本体の Resource IR match 実行意味論は、これに加えて unreachable arm 判定、raw cell loaded value origin、pending realloc result、pending variant initialization、variant match refinement を arm entry state に反映している。
- return summary は caller 側へ memory-safety fact を伝播するため、match payload の解釈が本体 checker とずれると、将来の Resource IR lowering 追加で summary fact の過不足が起きる。

## 問題

Return-transfer collection constructs match arm state with only raw/function alias and collection slot transfer, while the generic Resource IR match semantics also handle arm reachability, pending variant initialization, pending realloc result copies, and loaded value origin transfer.

## 影響

Collection slot return summaries can diverge from the initialized/move checker semantics around match payloads, causing false summary facts or missed owner/slot transfer evidence as Resource IR match lowering grows.

## 修正方針

Build match-arm entry state from the same generic Resource IR state components: skip unreachable arms, propagate raw/cell origin/function/pending realloc/variant state for owned payload binds, and apply variant match refinements before collecting arm returns.

## 修正内容

- return-transfer 収集時の match arm entry state を `Option<CollectionSlotSummaryBuildState>` にし、`PendingVariantRawCellInitializations::match_arm_reachable` が false の arm は収集対象から外すようにした。
- owned payload bind では `summary_check_engine` を使い、raw owner alias と cell rekey を同じ state transition で処理するようにした。
- bind payload から raw cell loaded value origin、collection slot state、function alias、pending realloc、pending variant initialization を bind local へ伝播するようにした。
- arm 本体へ入る前に `PendingVariantRawCellInitializations::apply_match_arm` を適用し、variant-specific raw initialization/refinement を return-transfer 収集側にも反映するようにした。
- 旧 `function_aliases_for_match_arm` helper は責務が狭くなり未使用になったため削除した。

## 検証

Add or retain regressions for match-bound collection slot return transfer and run collection slot call summary tests plus cargo check.

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
