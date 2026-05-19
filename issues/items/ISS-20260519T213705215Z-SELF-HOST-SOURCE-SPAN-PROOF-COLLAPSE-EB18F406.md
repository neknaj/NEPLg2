---
id: ISS-20260519T213705215Z-SELF-HOST-SOURCE-SPAN-PROOF-COLLAPSE-EB18F406
title: "Self-host source span proof collapses typed refutation to bool"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260519T213705215Z-SELF-HOST-SOURCE-SPAN-PROOF-COLLAPSE-EB18F406: Self-host source span proof collapses typed refutation to bool

## 概要

source span validity is represented as a typed proof result, but the public helper and module checker collapse it to bool and rebuild diagnostics manually. This keeps one proof path outside the evidence/refutation model.

## 対象

- `stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl`

## 根拠

- `SelfhostProofResult` は `Proven(SelfhostProofEvidence)` / `Refuted(SelfhostProofRefutation)` の typed enum だが、`selfhost_proof_source_span_valid` だけが `bool` を返していた。
- `check/module.nepl` は bool false branch で `ModuleItemSpanInvalid` diagnostic を手作りしており、`SelfhostProofRefutation::SourceSpanInvalid` を通していなかった。
- raw backend / module directive transition は typed refutation を checker diagnostic へ変換する形に揃えたため、source span だけ例外を残すと proof boundary の設計が二重化する。

## 問題

source span validity is represented as a typed proof result, but the public helper and module checker collapse it to bool and rebuild diagnostics manually. This keeps one proof path outside the evidence/refutation model.

## 影響

future checker code can copy the bool predicate pattern, losing refutation payloads and weakening the auditability of the generic proof engine before type/effect/resource obligations are added.

## 修正方針

Make source span validity return Result<(), SelfhostProofRefutation>, remove the generic bool helper, and update check/module.nepl plus doctests/source policies to match typed proof refutations.

## 検証

Run selfhost proof/checker doctests, source-policy contracts, issue check, and git diff check.

## 修正内容

- `selfhost_proof_source_span_valid` を `bool` ではなく `Result<(), SelfhostProofRefutation>` を返す API に変更した。
- `selfhost_proof_result_is_proven` を削除し、proof layer が typed evidence/refutation を bool に潰す public helper を持たないようにした。
- `check/module.nepl` に `selfhost_module_check_item_span` を追加し、source span proof の typed refutation を `selfhost_module_check_refutation_diag` へ渡すようにした。
- `tests/stdlib/neplg2_proof.n.md` と `stdlib/neplg2/core/proof.nepl` の doctest を typed Result を match する形へ更新した。
- `nodesrc/test_selfhost_proof_entry_contract.js` に、source span proof が `Result<(), SelfhostProofRefutation>` を返すこと、module checker が `match` で proof result を受けること、bool helper が復活しないことを追加した。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/test_selfhost_checker_report_contract.js`: pass
- `node nodesrc/test_selfhost_diag_code_enum.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-selfhost-source-span-proof-result-proof-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-source-span-proof-result-module-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-source-span-proof-result-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-source-span-proof-result-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
