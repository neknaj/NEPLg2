---
id: ISS-20260520T003842270Z-SELF-HOST-TRAIT-COHERENCE-IS-NOT-A-G-E107D307
title: "self-host trait coherence is not a generic proof obligation"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/ty/trait_ref.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_trait_proof.n.md"
---

# ISS-20260520T003842270Z-SELF-HOST-TRAIT-COHERENCE-IS-NOT-A-G-E107D307: self-host trait coherence is not a generic proof obligation

## 概要

SelfhostProofDomain::Trait exists, but trait impl overlap/coherence has no typed fact, obligation, evidence, or refutation, so later abstraction checking can grow checker-local rules.

## 対象

- `stdlib/neplg2/core/ty/trait_ref.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_trait_proof.n.md`

## 根拠

- `SelfhostProofDomain::Trait` は存在するが、`SelfhostProofFact` / `SelfhostProofObligation` / `SelfhostProofEvidence` / `SelfhostProofRefutation` に Trait domain の coherence variant がなかった。
- trait impl overlap を後続 checker が bool や trait 名文字列で直接判定できる状態だと、抽象化機能の安全性が checker-local rule に分散する。

## 問題

SelfhostProofDomain::Trait exists, but trait impl overlap/coherence has no typed fact, obligation, evidence, or refutation, so later abstraction checking can grow checker-local rules.

## 影響

Trait safety and generic abstraction work can bypass the generic proof solver, losing exhaustive match coverage and making overlapping impl regressions harder to audit before self-hosting.

## 修正方針

Add typed trait impl keys and relation facts derived from the source type arena, a TraitImplNonOverlapping obligation, typed evidence/refutation, a public proof wrapper, and focused doctests.

## 検証

Run the self-host proof contract test and focused trait proof doctests.

- 根本原因は、Trait domain が `SelfhostProofDomain` に予約されているだけで、source type arena から導出した impl key relation を generic proof solver へ渡す入口が欠けていたことだった。
- `core/ty/trait_ref.nepl` を追加し、`SelfhostTraitId`、`SelfhostTraitImplKey`、`SelfhostTraitImplRelation` を typed model として定義した。relation は trait 名文字列や stdlib module allowlist ではなく、trait id と `SelfhostTypeArena` の構造的 self type equality から計算する。
- `SelfhostTraitImplPairFact`、`SelfhostProofObligation::TraitImplNonOverlapping`、`SelfhostProofEvidence::TraitImplNonOverlapping`、`SelfhostProofRefutation::TraitImplCoherenceInvalid` を追加した。
- duplicate impl、invalid candidate key、invalid existing key は `SelfhostTraitImplCoherenceError` enum と issue payload に残し、caller が match で診断へ変換できる形にした。
- `selfhost_proof_trait_impl_non_overlapping` は public typed wrapper だが、内部では generic `selfhost_proof_solve` を通す。trait checker 側に個別証明器を増やさない。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/ty/trait_ref.nepl --no-tree -o tmp/agent1-trait-ref-model.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-trait-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-trait-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-trait-proof-existing-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-trait-proof-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
