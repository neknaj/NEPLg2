---
id: ISS-20260520T011528599Z-SELF-HOST-LIFETIME-OUTLIVES-IS-NOT-R-6C9E1A5D
title: "self-host lifetime outlives is not represented as a generic proof obligation"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/resource/lifetime.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_lifetime_proof.n.md"
---

# ISS-20260520T011528599Z-SELF-HOST-LIFETIME-OUTLIVES-IS-NOT-R-6C9E1A5D: self-host lifetime outlives is not represented as a generic proof obligation

## 概要

self-host proof boundary covers type kind, trait coherence, effect boundary, Resource cell state, and borrow access, but lifetime outlives proof is still not a typed fact/obligation/evidence/refutation domain.

## 対象

- `stdlib/neplg2/core/resource/lifetime.nepl`
- `stdlib/neplg2/core/proof/**`
- `tests/stdlib/neplg2_lifetime_proof.n.md`

## 根拠

- borrow access proof can reject simultaneous shared/mutable access, but it cannot reject a borrow escaping into a return value, longer-lived storage, or closure capture without a typed lifetime relation.
- If lifetime proof is left to future checker-local code, self-host static checking will grow ad hoc escape rules outside the generic proof solver.

## 問題

Lifetime escape safety must be represented as source-derived lifetime relation facts matched against `LifetimeOutlives` obligations. The current proof domain has no such typed model, so lifetime errors cannot be returned as structured proof refutations.

## 影響

Borrow/lifetime safety work can accept escaping short-lived borrows or duplicate lifetime checks in checker-specific code, weakening the Resource IR proof architecture before self-hosting.

## 修正方針

Add typed lifetime id/position/relation/use-kind models, `LifetimeOutlives` fact and obligation, evidence/refutation payloads, a public proof wrapper, source policy coverage, and focused doctests.

## 検証

Run the self-host proof contract test and focused lifetime proof doctests.

- 根本原因は、self-host proof boundary に borrow access state はあっても、borrow が return/storage/capture へ逃げる時の lifetime outlives 関係を typed fact / obligation として表す入口がなかったことだった。
- `core/resource/lifetime.nepl` を追加し、`SelfhostLifetimeId`、`SelfhostLifetimePosition`、`SelfhostLifetimeScopePathKind`、`SelfhostLifetimeRelation`、`SelfhostLifetimeUseKind` を typed model として定義した。scope depth だけでは siblings を区別できないため、outlives relation は source scope graph の typed path evidence から作る。
- `SelfhostProofDomain::Lifetime`、`SelfhostLifetimeOutlivesFact`、`SelfhostProofObligation::LifetimeOutlives`、`SelfhostProofEvidence::LifetimeOutlives`、`SelfhostProofRefutation::LifetimeOutlivesInvalid` を追加した。
- required lifetime mismatch、invalid subject/required lifetime、短い lifetime の escape、unrelated lifetime は `SelfhostLifetimeOutlivesError` enum と issue payload に残す。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/resource/lifetime.nepl --no-tree -o tmp/agent1-lifetime-model.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_lifetime_proof.n.md --no-tree -o tmp/agent1-lifetime-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-lifetime-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-lifetime-existing-proof-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_borrow_proof.n.md --no-tree -o tmp/agent1-lifetime-borrow-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_effect_proof.n.md --no-tree -o tmp/agent1-lifetime-effect-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-lifetime-trait-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_type_proof.n.md --no-tree -o tmp/agent1-lifetime-type-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-lifetime-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
