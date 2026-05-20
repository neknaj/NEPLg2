---
id: ISS-20260520T020822877Z-SELF-HOST-PROOF-SOLVER-IS-RE-GROWING-905DDB3F
title: "Self-host proof solver is re-growing as a flat proof surface"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/api.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260520T020822877Z-SELF-HOST-PROOF-SOLVER-IS-RE-GROWING-905DDB3F: Self-host proof solver is re-growing as a flat proof surface

## 概要

The self-host generic proof solver now owns both the internal proof rules and the public typed convenience API. As owner/resource/type/trait/effect proofs are added this recreates the flat Rust-compiler-style structure that the self-host layout is explicitly trying to avoid, and it weakens reviewability around which functions are the proof engine versus which functions are caller-facing adapters.

## 対象

- `stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/api.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `stdlib/neplg2/core/proof/solver.nepl` が typed wrapper と証明規則の両方を持ち、owner / borrow / lifetime / type / trait / effect proof の追加後に 1300 行超の flat module へ戻っていた。
- `nodesrc/test_selfhost_proof_entry_contract.js` は solver の public function surface を監視していたが、public typed wrapper と内部 proof rule の置き場を区別していなかった。

## 問題

The self-host generic proof solver now owns both the internal proof rules and the public typed convenience API. As owner/resource/type/trait/effect proofs are added this recreates the flat Rust-compiler-style structure that the self-host layout is explicitly trying to avoid, and it weakens reviewability around which functions are the proof engine versus which functions are caller-facing adapters.

## 影響

Static-check proof code becomes harder to audit and new domains may accidentally add checker-local or public ad hoc proof entry points instead of going through the generic proof boundary. This is a design risk for the large static-check rewrite because the proof program itself should stay structurally easy to inspect by contract tests.

## 修正方針

Split public typed convenience constructors and wrappers into proof/api.nepl while keeping proof/solver.nepl focused on SelfhostProofQuery -> SelfhostProofResult and private enum-matched proof rules. Keep mismatch construction generic and typed, and update contract tests so direct proof-rule helpers do not become public API.

## 対応内容

- `proof/api.nepl` を追加し、caller 向け typed wrapper と query builder を `proof/solver.nepl` から分離した。
- `proof/solver.nepl` は `selfhost_proof_solve(SelfhostProofQuery)` と private な domain proof rule に限定した。
- mismatch refutation 生成は `proof/query.nepl` の generic helper に移し、solver と API wrapper が同じ typed mismatch construction を共有するようにした。
- `proof.nepl` facade は fact / obligation / query / solver / api を re-export し、使用側の import surface は維持しつつ責務だけを分割した。
- 契約テストで solver の public API が `selfhost_proof_solve` だけであること、typed wrapper が `proof/api.nepl` にだけ存在すること、API wrapper が generic solver を経由することを固定した。

## 検証

- `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-proof-api-split-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md -i tests/stdlib/neplg2_owner_proof.n.md -i tests/stdlib/neplg2_borrow_proof.n.md -i tests/stdlib/neplg2_lifetime_proof.n.md -i tests/stdlib/neplg2_effect_proof.n.md -i tests/stdlib/neplg2_type_proof.n.md -i tests/stdlib/neplg2_trait_proof.n.md --no-tree -o tmp/agent1-proof-api-split-related.json -j 2 --dist web/dist --assert-io`: 12/12 passed
