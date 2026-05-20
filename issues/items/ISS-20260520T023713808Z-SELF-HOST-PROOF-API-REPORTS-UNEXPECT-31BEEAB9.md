---
id: ISS-20260520T023713808Z-SELF-HOST-PROOF-API-REPORTS-UNEXPECT-31BEEAB9
title: "Self-host proof API reports unexpected evidence as fact obligation mismatch"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/api.nepl, stdlib/neplg2/core/proof/query.nepl, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260520T023713808Z-SELF-HOST-PROOF-API-REPORTS-UNEXPECT-31BEEAB9: Self-host proof API reports unexpected evidence as fact obligation mismatch

## 概要

The self-host proof API wrapper match arms convert any proven evidence variant other than the expected one into FactObligationMismatch. That refutation describes fact/obligation domain wiring, not a solver evidence-kind bug, so a proof implementation error is classified as the wrong typed failure.

## 対象

- `stdlib/neplg2/core/proof/api.nepl, stdlib/neplg2/core/proof/query.nepl, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `proof/api.nepl` の public typed wrapper は `selfhost_proof_solve` から返った `SelfhostProofEvidence` を wrapper ごとに projection するが、期待外の evidence variant を `FactObligationMismatch` に変換していた。
- `FactObligationMismatch` は fact domain と obligation domain の接続誤りを表す refutation であり、domain が一致した query に対して solver が別種の成功 evidence を返した場合の proof-program error とは意味が異なる。
- `SelfhostProofRefutation` に variant を増やすと既存 doctest の exhaustive match が落ちるため、テスト側でも新分類を明示 arm として扱う必要があることが確認できた。

## 問題

The self-host proof API wrapper match arms convert any proven evidence variant other than the expected one into FactObligationMismatch. That refutation describes fact/obligation domain wiring, not a solver evidence-kind bug, so a proof implementation error is classified as the wrong typed failure.

## 影響

Static-check proof regressions can be hidden behind an unrelated mismatch payload. As evidence variants grow, API wrappers repeat the wrong refutation and make proof-program bugs harder to audit with match-based diagnostics.

## 修正方針

Add a typed SelfhostProofEvidenceKind and UnexpectedEvidence refutation. Derive expected evidence kind from the typed obligation and actual kind from the evidence, and make API wrappers return UnexpectedEvidence for mismatched proven evidence.

## 対応内容

- `SelfhostProofEvidenceKind` を追加し、`SelfhostProofEvidence` と `SelfhostProofObligation` の双方から kind を導出する関数を exhaustive match で実装した。
- `SelfhostProofUnexpectedEvidence` と `SelfhostProofRefutation::UnexpectedEvidence` を追加し、expected / actual を enum payload として保持するようにした。
- `proof/api.nepl` の期待外 evidence arm は `FactObligationMismatch` ではなく `selfhost_proof_query_unexpected_evidence_refutation` を返すようにした。
- module checker と proof doctest の `SelfhostProofRefutation` match に `UnexpectedEvidence` arm を追加し、新 variant の扱いを曖昧にしない回帰テストへ更新した。

## 検証

- `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-proof-unexpected-evidence-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-proof-unexpected-evidence-module.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md -i tests/stdlib/neplg2_owner_proof.n.md -i tests/stdlib/neplg2_borrow_proof.n.md -i tests/stdlib/neplg2_lifetime_proof.n.md -i tests/stdlib/neplg2_effect_proof.n.md -i tests/stdlib/neplg2_type_proof.n.md -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-proof-unexpected-evidence-related.json -j 2 --dist web/dist --assert-io`: 12/12 passed
