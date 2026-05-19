---
id: ISS-20260519T212225940Z-SELF-HOST-MODULE-DIRECTIVE-MULTIPLIC-33C1FBE9
title: "Self-host module directive multiplicity remains checker-local count"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260519T212225940Z-SELF-HOST-MODULE-DIRECTIVE-MULTIPLIC-33C1FBE9: Self-host module directive multiplicity remains checker-local count

## 概要

self-host module checker only records #entry/#target counts in SelfhostModuleCheckSummary and does not prove the module stream invariant that file-scoped singleton directives are unique. This leaves future checker logic likely to branch on summary counts rather than a typed proof obligation.

## 対象

- `stdlib/neplg2/core/proof/**, stdlib/neplg2/core/check/module.nepl`

## 根拠

- `ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062` で `core/proof/` の入口を作った後も、`SelfhostModuleCheckSummary` の `entry_count` / `target_count` は集計だけで、module stream が singleton directive の一意性を証明していなかった。
- Rust 側 compiler は `#target` の重複を loader/precheck 境界で拒否している。self-host 側も同じ file-scoped singleton invariant を S3 module checker 境界で保持する必要がある。
- このまま checker 側で summary count の `> 1` 判定を足すと、ユーザー方針に反して proof が個別 checker-local control flow として増える。

## 問題

self-host module checker only records #entry/#target counts in SelfhostModuleCheckSummary and does not prove the module stream invariant that file-scoped singleton directives are unique. This leaves future checker logic likely to branch on summary counts rather than a typed proof obligation.

## 影響

duplicate #target or #entry items can pass the self-host module validation boundary, and the design keeps module well-formedness as ad hoc checker state instead of the generic proof engine required by the static-check redesign.

## 修正方針

Add a typed module directive stream fact/state/obligation/result to core/proof, update check/module.nepl to drive #entry/#target multiplicity through that proof, add diagnostics and regression tests, and keep checker logic limited to mapping typed refutations into diagnostics.

## 検証

Run selfhost proof/checker doctests and source-policy contracts.

## 修正内容

- `SelfhostModuleDirectiveKind` / `SelfhostModuleDirectiveFact` を追加し、module item stream 上の `#entry` / `#target` 観測を proof fact にした。
- `SelfhostModuleDirectiveState` を `NoneSeen` / `EntrySeen` / `TargetSeen` / `EntryAndTargetSeen` の enum state として追加し、観測済み directive の span を state payload に保持するようにした。
- `SelfhostProofObligation::ModuleDirectiveTransition`、`SelfhostProofEvidence::ModuleDirectiveTransition`、`SelfhostProofRefutation::ModuleDirectiveDuplicate` を追加し、singleton directive multiplicity を proof solver の exhaustive match で検査する構造にした。
- `check/module.nepl` は `SelfhostModuleItemKind` から `SelfhostModuleDirectiveKind` への写像と、typed refutation から diagnostic への変換だけを担当するようにした。summary count による後処理は導入していない。
- `SelfhostCheckerDiagnosticCode::ModuleDirectiveDuplicate` を追加し、内部診断 ID は enum、表示用 stable string は `selfhost_checker_diag_code_name` の match だけで生成する形を維持した。
- `nodesrc/test_selfhost_proof_entry_contract.js` と `tests/stdlib/neplg2_{proof,checker}.n.md` を更新し、proof layer ownership と重複 `#entry` / `#target` rejection を回帰テスト化した。

## 検証結果

- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/test_selfhost_checker_report_contract.js`: pass
- `node nodesrc/test_selfhost_diag_code_enum.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md --no-tree -o tmp/agent1-selfhost-module-directive-proof-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-module-directive-checker-nmd.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/neplg2/core/proof.nepl --no-tree -o tmp/agent1-selfhost-module-directive-proof-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl --no-tree -o tmp/agent1-selfhost-module-directive-module-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/diag.nepl --no-tree -o tmp/agent1-selfhost-module-directive-diag-source.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
