---
id: ISS-20260519T215154303Z-SELF-HOST-PROOF-SOLVER-EXPOSES-INTER-38516C0B
title: "Self-host proof solver exposes internal proof rules as public API"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/solver.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260519T215154303Z-SELF-HOST-PROOF-SOLVER-EXPOSES-INTER-38516C0B: Self-host proof solver exposes internal proof rules as public API

## 概要

core/proof/solver.nepl keeps domain-specific proof rule helpers public, so callers can bypass the generic SelfhostProofQuery -> SelfhostProofResult entry point and depend on individual module/source rules directly.

## 対象

- `stdlib/neplg2/core/proof/solver.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `stdlib/neplg2/core/proof/solver.nepl` は `SelfhostProofQuery -> SelfhostProofResult` の generic entry point を持つ一方で、`selfhost_proof_solve_source_span_valid`、`selfhost_proof_solve_raw_backend_transition`、`selfhost_proof_solve_module_directive_transition` などの proof rule helper も `pub fn` として公開していた。
- `stdlib/neplg2/core/proof.nepl` は solver module を `pub #import` しているため、これらの helper は self-host proof facade から利用者に見える。
- helper が公開されたままだと、後続の type / effect / owner / lifetime / Resource IR proof が `SelfhostProofQuery` / `SelfhostProofResult` 境界ではなく、domain-local rule へ直接依存する設計に戻りやすい。

## 問題

core/proof/solver.nepl keeps domain-specific proof rule helpers public, so callers can bypass the generic SelfhostProofQuery -> SelfhostProofResult entry point and depend on individual module/source rules directly.

## 影響

Future type, effect, owner, lifetime, and Resource IR checks can grow checker-local or domain-local proof calls instead of going through the auditable typed proof boundary.

## 修正方針

Keep only the generic solver entry and intentional typed convenience wrappers public; make domain rule helpers private and add a source-policy regression that rejects public selfhost_proof_solve_* helpers other than selfhost_proof_solve.

## 検証

Run selfhost proof source/checker doctests, source-policy contracts, issue check, and git diff check.

## 修正内容

- `selfhost_proof_solve`、`selfhost_proof_source_span_valid`、`selfhost_proof_raw_backend_transition`、`selfhost_proof_module_directive_transition` だけを solver の public API として残した。
- source span / raw backend / module directive の proof rule helper と query builder は private `fn` に変更し、facade 外から個別 rule を直接呼べないようにした。
- `nodesrc/test_selfhost_proof_entry_contract.js` に solver public API allowlist を追加し、意図しない `pub fn` の再導入を拒否するようにした。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-selfhost-proof-internal-rules-proof-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-proof-internal-rules-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-proof-internal-rules-module-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-proof-internal-rules-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
