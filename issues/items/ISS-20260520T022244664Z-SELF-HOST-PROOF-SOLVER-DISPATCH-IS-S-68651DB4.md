---
id: ISS-20260520T022244664Z-SELF-HOST-PROOF-SOLVER-DISPATCH-IS-S-68651DB4
title: "Self-host proof solver dispatch is still quadratic across domains"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/fact.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260520T022244664Z-SELF-HOST-PROOF-SOLVER-DISPATCH-IS-S-68651DB4: Self-host proof solver dispatch is still quadratic across domains

## 概要

The self-host generic proof solver matches every obligation against every fact variant and repeats the same mismatch result across domains. This keeps the proof solver generic, but the dispatch shape grows as fact variants times obligation variants and makes it harder to audit whether a new domain was connected intentionally or only copied into mismatch arms.

## 対象

- `stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/fact.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `stdlib/neplg2/core/proof/solver.nepl` の public `selfhost_proof_solve` が obligation ごとに全 fact variant を直接 match し、domain mismatch も各 proof rule の中に繰り返し埋め込んでいた。
- `SelfhostProofDomain` は fact / obligation の分類として存在していたが、solver entry で fact domain と obligation domain を照合する typed helper がなかった。
- `nodesrc/test_selfhost_proof_entry_contract.js` は solver が enum match を使うことは監視していたが、public entry が domain dispatch を中央化することまでは固定していなかった。

## 問題

The self-host generic proof solver matches every obligation against every fact variant and repeats the same mismatch result across domains. This keeps the proof solver generic, but the dispatch shape grows as fact variants times obligation variants and makes it harder to audit whether a new domain was connected intentionally or only copied into mismatch arms.

## 影響

Static-check proof additions can reintroduce copy-pasted mismatch arms and hide proof wiring mistakes inside large match blocks. The solver program itself should make domain routing explicit, while still using exhaustive enum matches for each domain.

## 修正方針

Add an exhaustive SelfhostProofDomain equality helper and route SelfhostProofQuery through a domain precheck before entering domain-specific internal dispatch functions. Keep all proof rules under the single generic solver entry point and keep same-domain fact/obligation mismatches as typed FactObligationMismatch payloads.

## 対応内容

- `SelfhostProofDomain` の比較を `selfhost_proof_domain_eq` に集約し、Source / Module / Type / Trait / Lifetime / Owner / Effect / Resource を wildcard なしの網羅 `match` で列挙した。
- public `selfhost_proof_solve` は fact / obligation から domain を導出し、domain が一致した場合だけ private `selfhost_proof_solve_matching_domain` に dispatch する構造へ分割した。
- domain mismatch は各 proof rule へ入る前に `selfhost_proof_query_mismatch_result` として typed `FactObligationMismatch` payload へ落とす。same-domain 内の fact/obligation mismatch は、既存どおり enum match の中で明示的に refutation へ変換する。
- 契約テストで domain equality が全 domain variant を列挙すること、public solver が typed domain precheck を経由すること、内部 dispatch が obligation/fact の enum match と wildcard 禁止を維持することを固定した。

## 検証

- `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-proof-domain-dispatch-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md -i tests/stdlib/neplg2_owner_proof.n.md -i tests/stdlib/neplg2_borrow_proof.n.md -i tests/stdlib/neplg2_lifetime_proof.n.md -i tests/stdlib/neplg2_effect_proof.n.md -i tests/stdlib/neplg2_type_proof.n.md -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-proof-domain-dispatch-related.json -j 2 --dist web/dist --assert-io`: 12/12 passed
