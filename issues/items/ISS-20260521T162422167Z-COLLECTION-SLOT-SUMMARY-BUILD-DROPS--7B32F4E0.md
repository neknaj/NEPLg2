---
id: ISS-20260521T162422167Z-COLLECTION-SLOT-SUMMARY-BUILD-DROPS--7B32F4E0
title: "Collection slot summary build drops branch range facts before traversal proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_build_ops.rs
---

# ISS-20260521T162422167Z-COLLECTION-SLOT-SUMMARY-BUILD-DROPS--7B32F4E0: Collection slot summary build drops branch range facts before traversal proof

## 概要

Collection slot summary build collects nested branch summaries from the pre-branch state and does not apply the branch condition_fact to then/else summary states. A callee that proves symbolic collection slot traversal under a typed range guard can pass local Resource IR checking but fail to emit a certified summary traversal proof.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_ops.rs`

## 根拠

- `ResourceOp::Branch` の summary 収集は then/else を pre-branch state から収集しており、`condition_fact` を truthy/falsey path の `RawCellAddressAliases` に反映していなかった。
- `ResourceOp::Loop` の summary 収集も `condition_ops` 後の state と truthy `condition_fact` を body summary に渡していなかった。
- 本体の initialized checker は同じ `ResourceConditionFact` を path-sensitive に反映しているため、callee 本体で成立する symbolic range proof が summary build では成立しない不整合があった。

## 問題

Collection slot summary build collects nested branch summaries from the pre-branch state and does not apply the branch condition_fact to then/else summary states. A callee that proves symbolic collection slot traversal under a typed range guard can pass local Resource IR checking but fail to emit a certified summary traversal proof.

## 影響

Non-Copy collection cleanup helpers cannot reliably carry source-derived range proofs through function summaries. This blocks self-host collection APIs and can push the design toward stdlib helper allowlists or marker-only cleanup summaries.

## 修正方針

Apply ResourceConditionFact value constraints to branch-specific CollectionSlotSummaryBuildState before collecting nested summary ops, mirroring the initialized checker semantics. Keep then/else facts path-sensitive and do not add wildcard handling.

## 対応内容

- Branch summary collection now applies the condition fact to then/else path states before collecting nested summary ops.
- Loop summary collection now evaluates `condition_ops` into a path state and applies the truthy condition fact before collecting body summary ops.
- Added a regression that a guarded symbolic collection slot drop traversal emits a certified summary traversal only when the branch condition facts are available.
- Follow-up issue for caller-side symbolic operand instantiation: `ISS-20260521T163555637Z-COLLECTION-SLOT-SUMMARY-CANNOT-INSTA-02D58E62`.

## 検証

- `cargo test -p nepl-core --lib collection_slot_summary_build_ops -- --test-threads=1`
- `collection_slot_summary_branch_condition_fact_certifies_symbolic_drop_traversal` verifies that guarded symbolic traversal summary certification receives `NonNegative` and `index < initialized_count` branch facts.
