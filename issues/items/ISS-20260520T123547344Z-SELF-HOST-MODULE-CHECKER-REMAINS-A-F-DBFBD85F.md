---
id: ISS-20260520T123547344Z-SELF-HOST-MODULE-CHECKER-REMAINS-A-F-DBFBD85F
title: "self-host module checker remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/module/summary.nepl, stdlib/neplg2/core/check/module/summary_update.nepl, stdlib/neplg2/core/check/module/diagnostic.nepl, stdlib/neplg2/core/check/module/raw_backend_adapter.nepl, stdlib/neplg2/core/check/module/declaration_adapter.nepl, stdlib/neplg2/core/check/module/orchestrate.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260520T123547344Z-SELF-HOST-MODULE-CHECKER-REMAINS-A-F-DBFBD85F: self-host module checker remains a flat implementation file

## 概要

The self-host module checker keeps summary storage/accessors, summary update, proof-to-diagnostic mapping, raw backend proof adapter, directive/declaration proof adapter, and orchestration loop in one file. That invites checker-local proof shortcuts as static checking grows.

## 対象

- `stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/module/summary.nepl, stdlib/neplg2/core/check/module/summary_update.nepl, stdlib/neplg2/core/check/module/diagnostic.nepl, stdlib/neplg2/core/check/module/raw_backend_adapter.nepl, stdlib/neplg2/core/check/module/declaration_adapter.nepl, stdlib/neplg2/core/check/module/orchestrate.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- `stdlib/neplg2/core/check/module.nepl` が `SelfhostModuleCheckSummary`、summary update、proof refutation -> diagnostic 変換、raw backend adapter、directive/declaration/span adapter、実行 loop を 443 行の単一 file に保持していた。
- module checker は今後 type / lifetime / effect / Resource IR の entry に近い層なので、ここが flat なままだと checker-local proof shortcut や public helper の漏出を source policy で監視しにくい。
- 既存の proof boundary は generic solver へ集約する方針であり、module checker 側は typed fact producer / refutation-to-diagnostic adapter / orchestration に分かれているべきだった。

## 問題

The self-host module checker keeps summary storage/accessors, summary update, proof-to-diagnostic mapping, raw backend proof adapter, directive/declaration proof adapter, and orchestration loop in one file. That invites checker-local proof shortcuts as static checking grows.

## 影響

Future type/lifetime/effect/resource checks can accrete into the same flat file, weakening the generic proof-boundary design and making public API/policy audits harder.

## 修正方針

Turn core/check/module.nepl into a facade. Split summary, summary update, diagnostic mapping, raw backend proof adapter, declaration/directive/span proof adapter, and orchestration into separate submodules. Keep only the current public summary API and selfhost_check_module_ast visible through the facade.

## 検証

Run self-host proof entry policy, module checker split policy, and focused checker doctests.

## 2026-05-20 Agent 1 修正

`stdlib/neplg2/core/check/module.nepl` を doctest と public re-export だけを持つ facade にした。実装は `module/summary.nepl`、`module/summary_update.nepl`、`module/diagnostic.nepl`、`module/raw_backend_adapter.nepl`、`module/declaration_adapter.nepl`、`module/orchestrate.nepl` に分割した。

設計上の境界:

- `summary.nepl`: public summary 型と accessor のみ。proof / diagnostic へ依存しない。
- `summary_update.nepl`: `SelfhostModuleItemKind` の exhaustive match による集計更新のみ。
- `raw_backend_adapter.nepl`: module item から raw backend fact を作り、generic proof API の typed `SelfhostProofRefutation` を返す。
- `declaration_adapter.nepl`: source span、singleton directive、declaration header の proof adapter。diagnostic へ直結しない。
- `diagnostic.nepl`: typed refutation から checker diagnostic への変換を集約。
- `orchestrate.nepl`: adapter の typed result を順に match し、診断変換と summary 更新を行う。

`nodesrc/test_selfhost_proof_entry_contract.js` は分割後の source 群を読む形に更新し、`nodesrc/test_selfhost_module_checker_split_contract.js` を追加して facade への実装再導入、summary/proof/diagnostic の責務混入、adapter が `SelfhostDiagnostic` を返す退行、orchestration が proof solver や診断文字列を直接持つ退行を検出する。この contract は `nodesrc/run_source_policy_regressions.js` にも追加した。

subagent review で「facade の expected import は確認しているが、追加の `pub #import` を allowlist で拒否していない」と指摘されたため、`module.nepl` facade の public re-export が `./module/summary as *` と `./module/orchestrate as *` だけであることも両 contract で固定した。

検証:

- `node nodesrc/test_selfhost_module_checker_split_contract.js`: passed
- `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
- `node nodesrc/test_selfhost_checker_report_contract.js`: passed
- `node nodesrc/tests.js -i stdlib\neplg2\core\check\module.nepl -i stdlib\neplg2\core\check\checker.nepl ...`: compile timeout。detached HEAD `41c89210` でも `module.nepl` doctest が `NEPL_TEST_CASE_TIMEOUT_MS=90000` で同じ compile timeout になったため、今回の分割による新規退行ではない。別 issue `ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D` として分離した。
