---
id: ISS-20260520T001923700Z-SELF-HOST-TYPE-KIND-COMPATIBILITY-IS-8009C28D
title: "self-host type kind compatibility is not a generic proof obligation"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_type_proof.n.md"
---

# ISS-20260520T001923700Z-SELF-HOST-TYPE-KIND-COMPATIBILITY-IS-8009C28D: self-host type kind compatibility is not a generic proof obligation

## 概要

The self-host proof domain enum contains Type, but type-kind compatibility is still only available as a boolean helper, so later type checking can bypass typed evidence/refutation and generic proof wiring.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/proof/**, tests/stdlib/neplg2_type_proof.n.md`

## 根拠

- `SelfhostProofDomain::Type` は存在するが、`SelfhostProofFact` / `SelfhostProofObligation` / `SelfhostProofEvidence` / `SelfhostProofRefutation` に Type domain の variant がなかった。
- `selfhost_type_kind_eq` は enum exhaustive match ではあるが、結果を bool に潰す helper であり、checker が typed evidence/refutation を経由せず type compatibility を直接判定できる状態だった。

## 問題

The self-host proof domain enum contains Type, but type-kind compatibility is still only available as a boolean helper, so later type checking can bypass typed evidence/refutation and generic proof wiring.

## 影響

Type checker work can grow checker-local type rules and lose exhaustiveness, making type safety regressions harder to audit before self-hosting.

## 修正方針

Add a typed type-kind observation fact, TypeKindCompatible obligation, evidence/refutation payloads, a public typed wrapper, and focused doctests.

## 検証

Run the self-host proof contract test and focused type proof doctests.

- 根本原因は、self-host proof architecture に Type domain の enum はある一方で、type kind compatibility を `SelfhostProofQuery` として表す fact / obligation / evidence / refutation が欠けていたことだった。
- `SelfhostTypeKindFact` を追加し、観測した `SelfhostTypeKind` と source span を typed fact として保持するようにした。
- `SelfhostProofObligation::TypeKindCompatible`、`SelfhostProofEvidence::TypeKindCompatible`、`SelfhostProofRefutation::TypeKindMismatch` を追加し、type kind mismatch では expected / actual / span を `SelfhostTypeKindMismatch` に残す。
- `selfhost_proof_type_kind_compatible` を public typed wrapper として追加し、内部 query は generic `selfhost_proof_solve` を通す。成功・失敗とも caller が match できるため、type checker 側に checker-local bool 判定を積み増す必要がない。
- `nodesrc/test_selfhost_proof_entry_contract.js` に Type domain fact / obligation / evidence / refutation と public solver surface の監視を追加した。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_type_proof.n.md --no-tree -o tmp/agent1-type-proof-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-type-proof-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-type-proof-module.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_effect_proof.n.md --no-tree -o tmp/agent1-type-proof-effect-nmd.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-type-proof-existing-nmd.json -j 1 --dist web/dist --assert-io`: 6/6 passed
