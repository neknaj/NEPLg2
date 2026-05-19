---
id: ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062
title: "Self-host checker lacks a generic proof entry point"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof.nepl, stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062: Self-host checker lacks a generic proof entry point

## 概要

The self-host module checker still validates initial source properties directly and only documents a future core/proof layer. This lets new static-check work grow as checker-local logic instead of typed facts and obligations consumed by a shared proof solver.

## 対象

- `stdlib/neplg2/core/proof.nepl, stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は、self-host static checking の証明器を `core/proof/` に汎用基盤として置き、owner / initialized / borrow / effect / abstraction check が個別 ad hoc 証明器を持たないことを P1 として要求している。
- 実装前の `stdlib/neplg2/core/check/module.nepl` は module item span validity を直接 `source_span_is_valid item.span` で判定しており、今後の checker 拡張が module-local proof logic として増える入口になっていた。
- `stdlib/neplg2/core/proof/` は存在せず、typed fact / obligation / solver の最小境界もまだなかった。

## 問題

The self-host module checker still validates initial source properties directly and only documents a future core/proof layer. This lets new static-check work grow as checker-local logic instead of typed facts and obligations consumed by a shared proof solver.

## 影響

Self-host static checking can drift toward per-module proof code, making owner/initialized/borrow/effect/abstraction checks harder to audit and weakening the no-allowlist, enum/match-based verification policy before the compiler is self-hosted.

## 修正方針

Add the initial core/proof facade with typed fact, obligation, query/result, and solver modules. Wire module item span validation through the generic proof query so check/module acts as a fact/obligation producer rather than a local proof engine, and add a source-policy regression for this boundary.

## 検証

Run the selfhost proof boundary source policy, focused checker/proof doctests, and issue index validation.

## 修正内容

- `stdlib/neplg2/core/proof.nepl` と `core/proof/{fact,obligation,query,solver}.nepl` を追加し、proof domain、fact、obligation、query、result kind を enum / struct payload として定義した。
- 初期 solver は `SourceSpanObserved` fact と `SourceSpanValid` obligation を照合し、`match` による網羅分岐で source span validity を証明する。
- `stdlib/neplg2/core/check/module.nepl` は `source_span_is_valid item.span` を直接呼ばず、`selfhost_proof_source_span_valid item.span` を通すようにした。module checker は fact / obligation producer の利用側へ寄せ、`checker.nepl` は orchestration-only のままにした。
- `nodesrc/test_selfhost_proof_entry_contract.js` を追加し、proof facade、typed enum payload、solver の wildcard 禁止、module checker の proof 経由利用を監視する。
- `tests/stdlib/neplg2_proof.n.md` を追加し、valid / invalid span proof を stdout 付き doctest report で固定した。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/test_selfhost_checker_report_contract.js`: pass
- `node nodesrc/test_selfhost_diag_code_enum.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-proof-entry-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-selfhost-proof-entry-proof-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-proof-entry-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-proof-entry-module-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/checker.nepl --no-tree -o tmp/agent1-selfhost-proof-entry-checker-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1

## 残る作業

この issue は generic proof entry point の確立であり、type / trait / effect / Resource IR の full proof engine 完成ではない。次の self-host static-check 作業では、raw backend block 状態、declaration well-formedness、trait coherence、effect boundary、owner/initialized/borrow obligation を同じ `SelfhostProofFact` / `SelfhostProofObligation` domain に追加し、個別 checker-local proof を増やさない。
