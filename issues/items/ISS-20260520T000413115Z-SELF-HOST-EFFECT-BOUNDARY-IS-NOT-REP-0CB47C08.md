---
id: ISS-20260520T000413115Z-SELF-HOST-EFFECT-BOUNDARY-IS-NOT-REP-0CB47C08
title: "self-host effect boundary is not represented as a generic proof obligation"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/ty/effect.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_effect_proof.n.md"
---

# ISS-20260520T000413115Z-SELF-HOST-EFFECT-BOUNDARY-IS-NOT-REP-0CB47C08: self-host effect boundary is not represented as a generic proof obligation

## 概要

The self-host compiler has no typed effect model or proof obligation for pure, impure, unsafe, and internal allocation effects, so later effect checks would need checker-local rules or bool/string metadata.

## 対象

- `stdlib/neplg2/core/ty/effect.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_effect_proof.n.md`

## 根拠

- `core/proof` には Source / Module / Resource domain の fact / obligation はあるが、effect domain の typed fact / obligation / evidence / refutation がなく、後続の effect checker が bool/string flag や checker-local special-case を追加しやすい状態だった。
- `InternalAlloc` を pure context へ fold する条件も型として表現されておらず、no-escape proof と単なる allocation effect の区別が proof boundary に残らなかった。

## 問題

The self-host compiler has no typed effect model or proof obligation for pure, impure, unsafe, and internal allocation effects, so later effect checks would need checker-local rules or bool/string metadata.

## 影響

Pure-context and unsafe-memory effect safety cannot be extended without risking ad hoc allowlists or payload-free decisions that conflict with the Stage 6 proof architecture.

## 修正方針

Add typed effect and effect-context enums, route effect observations through the generic proof fact/obligation/evidence/refutation boundary, and test pure-context rejection plus internal-allocation no-escape folding.

## 検証

Run the self-host proof contract test and focused effect proof doctests.

- 根本原因は、self-host compiler 側に pure / impure / unsafe / internal allocation effect の typed model がなく、effect boundary を generic proof solver の obligation として扱えなかったことだった。
- `SelfhostEffectKind`、`SelfhostEffectEscapeState`、`SelfhostEffectContext` を追加し、effect observation fact が effect kind と no-escape proof state を型付きで保持できるようにした。
- `SelfhostProofFact::EffectObserved`、`SelfhostProofObligation::EffectAllowedInContext`、`SelfhostProofEvidence::EffectAllowed`、`SelfhostProofRefutation::EffectBoundaryInvalid` を追加し、effect boundary を Source / Module / Resource と同じ `SelfhostProofQuery` 経由に載せた。
- pure context では `Pure` と no-escape 証明済み `InternalAlloc` だけを許可し、`ExternalIo` / `Nondet` / unsafe memory / escape し得る internal allocation は typed reason 付き refutation にする。unsafe memory は `UnsafeBoundary` context でのみ許可する。
- `nodesrc/test_selfhost_proof_entry_contract.js` に effect fact / obligation / evidence / refutation と solver public surface の監視を追加した。これは stdlib module allowlist ではなく、proof layer の enum payload と public API 形状を source policy として固定する。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/ty/effect.nepl --no-tree -o tmp/agent1-effect-model.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_effect_proof.n.md --no-tree -o tmp/agent1-effect-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-effect-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-effect-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-effect-proof-existing-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
