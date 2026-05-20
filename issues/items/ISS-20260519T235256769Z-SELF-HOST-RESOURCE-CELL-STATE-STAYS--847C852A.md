---
id: ISS-20260519T235256769Z-SELF-HOST-RESOURCE-CELL-STATE-STAYS--847C852A
title: "self-host resource cell state stays outside the generic proof solver"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/resource/move_state.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_proof.n.md"
---

# ISS-20260519T235256769Z-SELF-HOST-RESOURCE-CELL-STATE-STAYS--847C852A: self-host resource cell state stays outside the generic proof solver

## 概要

The self-host resource move_state module is still only a Stage 0 marker, and initialized, moved, and dropped cell transitions are not represented as typed proof facts and obligations.

## 対象

- `stdlib/neplg2/core/resource/move_state.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_proof.n.md`

## 根拠

- 未記入

## 問題

The self-host resource move_state module is still only a Stage 0 marker, and initialized, moved, and dropped cell transitions are not represented as typed proof facts and obligations.

## 影響

Resource IR work would either add ad hoc checker-local state machines or rely on untyped conventions, which conflicts with the Stage 6 requirement that owner and initialized state be proven through the generic proof boundary.

## 修正方針

Add typed resource cell state and event enums, route resource cell transitions through SelfhostProofFact, SelfhostProofObligation, SelfhostProofEvidence, and typed refutations, and cover invalid move/drop transitions with doctests.

## 検証

Run the self-host proof contract test and focused resource/proof doctests.

- 根本原因は、`core/resource/move_state.nepl` が Stage 0 marker だけで、initialized / moved / dropped の cell state を typed model として持たず、proof solver へ渡す Resource domain fact / obligation もなかったことだった。
- `SelfhostResourceCellState` を `Uninitialized | Initialized | Moved | Dropped` の enum として追加し、`SelfhostResourceCellEventKind` を `Initialize | MoveOut | Drop` の enum として追加した。
- `SelfhostResourceCellEventFact`、`SelfhostProofObligation::ResourceCellTransition`、`SelfhostProofEvidence::ResourceCellTransition`、`SelfhostProofRefutation::ResourceCellTransitionInvalid` を追加し、Resource cell transition を generic proof solver 経由で検査するようにした。
- 不正遷移は `SelfhostResourceCellTransitionIssue` に state / event / span / reason を保持する。reason は `SelfhostResourceCellTransitionError` enum で、double drop や move after move を文字列ではなく typed payload として残す。
- `tests/stdlib/neplg2_proof.n.md` に initialize -> move -> drop-after-move の回帰テストを追加し、drop-after-move が typed refutation になることを確認した。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/resource/move_state.nepl --no-tree -o tmp/agent1-resource-cell-move-state.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-resource-cell-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-resource-cell-proof-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-resource-cell-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
