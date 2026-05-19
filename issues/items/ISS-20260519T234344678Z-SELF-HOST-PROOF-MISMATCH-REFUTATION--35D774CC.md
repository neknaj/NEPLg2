---
id: ISS-20260519T234344678Z-SELF-HOST-PROOF-MISMATCH-REFUTATION--35D774CC
title: "self-host proof mismatch refutation drops fact and obligation domains"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, tests/stdlib/neplg2_proof.n.md"
---

# ISS-20260519T234344678Z-SELF-HOST-PROOF-MISMATCH-REFUTATION--35D774CC: self-host proof mismatch refutation drops fact and obligation domains

## 概要

SelfhostProofRefutation::FactObligationMismatch is a payload-free variant, so the generic proof solver loses which fact domain and obligation domain were connected incorrectly.

## 対象

- `stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, tests/stdlib/neplg2_proof.n.md`

## 根拠

- 未記入

## 問題

SelfhostProofRefutation::FactObligationMismatch is a payload-free variant, so the generic proof solver loses which fact domain and obligation domain were connected incorrectly.

## 影響

As type, trait, effect, and resource obligations are added, proof program wiring mistakes become harder to diagnose and harder to audit through typed exhaustive matching.

## 修正方針

Add a typed mismatch payload carrying fact and obligation proof domains, route all generic mismatch construction through a helper, and update proof callers/tests to match the payload explicitly.

## 検証

Run the self-host proof contract test and focused neplg2 proof doctests.

- 根本原因は、`SelfhostProofRefutation::FactObligationMismatch` が payload-free であり、generic proof solver の fact / obligation 接続ミスを「何かが噛み合わなかった」という情報に潰していたことだった。
- `SelfhostProofMismatch` を追加し、fact domain と obligation domain を `SelfhostProofDomain` として保持するようにした。
- mismatch 生成は `selfhost_proof_fact_domain` / `selfhost_proof_obligation_domain` を使う private helper に集約した。stdlib 名や module 名の列挙ではなく、fact / obligation enum payload から domain を導出する。
- `SelfhostProofRefutation::FactObligationMismatch` は `SelfhostProofMismatch` payload 必須に変更したため、caller は match で payload を受ける必要がある。
- `tests/stdlib/neplg2_proof.n.md` に、raw backend fact と source span obligation を誤接続した query が `fact_domain=Module` / `obligation_domain=Source` を返す回帰テストを追加した。
- `nodesrc/test_selfhost_proof_entry_contract.js` に payload-free mismatch variant の再導入禁止と、mismatch domain 導出 helper の構造検査を追加した。
- focused verification:
  - `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-proof-mismatch-domains.json -j 1 --dist web/dist --assert-io`: 5/5 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-proof-mismatch-core-proof.json -j 1 --dist web/dist --assert-io`: 1/1 passed
  - `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-proof-mismatch-module-check.json -j 1 --dist web/dist --assert-io`: 1/1 passed
