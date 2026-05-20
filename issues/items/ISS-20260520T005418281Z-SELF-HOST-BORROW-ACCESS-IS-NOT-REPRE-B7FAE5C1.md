---
id: ISS-20260520T005418281Z-SELF-HOST-BORROW-ACCESS-IS-NOT-REPRE-B7FAE5C1
title: "self-host borrow access is not represented as a Resource proof obligation"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/resource/borrow_state.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_borrow_proof.n.md"
---

# ISS-20260520T005418281Z-SELF-HOST-BORROW-ACCESS-IS-NOT-REPRE-B7FAE5C1: self-host borrow access is not represented as a Resource proof obligation

## 概要

Resource domain proof currently covers initialized/moved/dropped cell transitions, but shared/mutable borrow access compatibility has no typed state, fact, obligation, evidence, or refutation.

## 対象

- `stdlib/neplg2/core/resource/borrow_state.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_borrow_proof.n.md`

## 根拠

- `SelfhostProofDomain::Resource` は initialized / moved / dropped cell transition だけを扱い、shared / mutable borrow の排他性を typed proof として表せなかった。
- borrow access の互換性が proof solver にないと、後続の borrow checker が alias rule を checker-local に実装する圧力が残る。

## 問題

Resource domain proof currently covers initialized/moved/dropped cell transitions, but shared/mutable borrow access compatibility has no typed state, fact, obligation, evidence, or refutation.

## 影響

Borrow safety work can grow checker-local alias rules and fail to reject mutable/shared conflicts through the generic proof solver before self-hosting.

## 修正方針

Add typed borrow state/request models, ResourceBorrowAccess facts and obligations, evidence/refutation payloads, a public proof wrapper, and focused doctests.

## 検証

Run the self-host proof contract test and focused borrow proof doctests.

- 根本原因は、Resource domain proof が cell state transition に留まり、borrow access state と request を enum payload として proof solver に渡す入口がなかったことだった。
- `core/resource/borrow_state.nepl` を追加し、`SelfhostBorrowState` と `SelfhostBorrowRequestKind` を typed model として定義した。
- `SelfhostBorrowAccessFact`、`SelfhostProofObligation::ResourceBorrowAccess`、`SelfhostProofEvidence::ResourceBorrowAccess`、`SelfhostProofRefutation::BorrowAccessInvalid` を追加した。
- shared borrow 中の mutable borrow、mutable borrow 中の shared borrow、invalid shared count、対応しない end request は `SelfhostBorrowAccessError` enum と issue payload に残す。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/resource/borrow_state.nepl --no-tree -o tmp/agent1-borrow-state-model.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_borrow_proof.n.md --no-tree -o tmp/agent1-borrow-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-borrow-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-borrow-proof-existing-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-borrow-proof-trait-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_type_proof.n.md --no-tree -o tmp/agent1-borrow-proof-type-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-borrow-proof-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_effect_proof.n.md --no-tree -o tmp/agent1-borrow-proof-effect-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
