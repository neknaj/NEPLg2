---
id: ISS-20260519T210448628Z-SELF-HOST-RAW-BACKEND-BLOCK-VALIDATI-FA9ABCD2
title: "Self-host raw backend block validation remains checker-local proof"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260519T210448628Z-SELF-HOST-RAW-BACKEND-BLOCK-VALIDATI-FA9ABCD2: Self-host raw backend block validation remains checker-local proof

## 概要

After the generic proof entry point was added, raw backend block state validation still lives in check/module as a module-local state machine. New static-check domains could copy this pattern and grow separate proof logic outside core/proof.

## 対象

- `stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062` で `core/proof/` の入口を作った直後でも、`check/module.nepl` には raw backend block の `Normal / empty / ready` state machine が残っていた。
- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は、個別 checker を証明器にせず、各 stage は typed fact と obligation を `core/proof/` に渡す方針を定めている。
- raw backend text/block consistency は module item stream の初期検査だが、今後 Resource IR / effect / trait obligation を追加する前に、checker-local proof pattern を残さない必要がある。

## 問題

After the generic proof entry point was added, raw backend block state validation still lives in check/module as a module-local state machine. New static-check domains could copy this pattern and grow separate proof logic outside core/proof.

## 影響

The self-host checker can drift away from the planned generic proof engine. Raw backend text/block consistency is still proved by checker-local control flow rather than a typed proof fact, obligation, evidence, and refutation model, weakening the auditability policy before resource/effect obligations are added.

## 修正方針

Represent raw backend item observation and transition state as typed proof facts/obligations, make the solver return typed evidence or typed refutation, and have check/module map proof refutations to checker diagnostics instead of owning the proof state machine.

## 検証

Run selfhost proof entry source policy, checker/proof doctests, and issue checks.

## 修正内容

- `SelfhostRawBackendKind` / `SelfhostRawBackendItemKind` / `SelfhostRawBackendItemFact` を proof fact 側に追加し、module item の raw backend 観測を typed fact として表すようにした。
- `SelfhostRawBackendState` / `SelfhostRawBackendOpenBlock` と `SelfhostProofObligation::RawBackendTransition` を追加し、raw backend block transition を proof obligation にした。
- `SelfhostProofResult` を `Proven(SelfhostProofEvidence)` / `Refuted(SelfhostProofRefutation)` の enum に変更し、成功時は next state evidence、失敗時は typed refutation を返す設計にした。これにより proven/refuted kind と payload の不整合が型で起きない。
- `selfhost_proof_solve_raw_backend_transition` と `selfhost_proof_raw_backend_transition` を追加し、raw text without block、empty raw block、stream end close を solver の exhaustive match へ移した。
- `check/module.nepl` から `SelfhostModuleRawState` と checker-local transition helper を削除し、AST item kind を `SelfhostRawBackendItemKind` に写したうえで proof solver へ渡す形にした。diagnostic 化だけは module checker が typed refutation を match して行う。
- `nodesrc/test_selfhost_proof_entry_contract.js` を更新し、raw backend fact / obligation / evidence / refutation が proof layer にあること、module checker が raw state enum を持たないことを監視する。
- `tests/stdlib/neplg2_proof.n.md` に raw backend transition の stdout doctest を追加した。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-raw-backend-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-selfhost-raw-backend-proof-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-raw-backend-module-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-raw-backend-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
