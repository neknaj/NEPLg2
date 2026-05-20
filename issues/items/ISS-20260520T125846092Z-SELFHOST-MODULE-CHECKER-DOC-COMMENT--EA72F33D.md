---
id: ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D
title: "selfhost module checker doc-comment doctests exceed compile timeout"
area: selfhost
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/checker.nepl"
---

# ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D: selfhost module checker doc-comment doctests exceed compile timeout

## 概要

The module/checker doc-comment doctests compile parser + selfhost checker paths and time out in the compile phase even when the case timeout is raised to 90000-180000ms. A detached HEAD worktree at 41c89210 reproduced the timeout, so the module checker split did not introduce the issue.

## 対象

- `stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/checker.nepl`

## 根拠

- 現在の作業ブランチで `node nodesrc\tests.js -i stdlib\neplg2\core\check\module.nepl --no-tree --dist web\dist -o tmp\agent1-selfhost-module-checker-split-module-180.json -j 1 --assert-io` を `NEPL_TEST_CASE_TIMEOUT_MS=180000` 付きで実行しても compile phase timeout になった。
- 同じ `module.nepl` doc-comment doctest は detached HEAD `41c89210` の worktree でも `NEPL_TEST_CASE_TIMEOUT_MS=90000` で compile timeout になった。
- import だけの smoke (`#import "neplg2/core/check/module" as *` して `0` を返す最小 source) は compile 約 12 秒で pass したため、file/directory split による import 循環ではなく、parser + checker fixture 本体の compile-time cost が支配的である。

## 問題

The module/checker doc-comment doctests compile parser + selfhost checker paths and time out in the compile phase even when the case timeout is raised to 90000-180000ms. A detached HEAD worktree at 41c89210 reproduced the timeout, so the module checker split did not introduce the issue.

## 影響

The doc-comment doctests cannot be used as a focused regression gate for the selfhost checker. Static-check performance regressions and genuine checker behavior regressions can be conflated unless the fixture is split or compiler compile-time cost is reduced.

## 修正方針

Investigate whether the broad parser+checker fixture, proof/checker monomorphization, Resource IR summary propagation, or unused imported selfhost code drives the timeout. Prefer compiler-side cost reduction or principled fixture decomposition with stdout assertions; do not solve by deleting coverage or merely raising the timeout.

## 修正

- `checker.nepl` の smoke API から `module_parser` import と `selfhost_parse_module_source` 呼び出しを削除した。
- `selfhost_checker_stage0` と `check/module.nepl` の doc-comment doctest は parser source string ではなく、`SelfhostModuleAst`、`SelfhostModuleDeclarationHeader`、`SelfhostModuleDeclarationHead` を直接構築する typed fixture にした。
- declaration item は `selfhost_module_item_new_with_declaration` を使い、header evidence 欠落を隠さず checker が本来受け取る AST boundary を検査する。
- `nodesrc/test_selfhost_module_checker_split_contract.js` に、checker/module の focused coverage が parser import / parse entry 呼び出しへ戻らないことを監視する contract を追加した。

## 根本原因

The selfhost checker focused doctests were unintentionally exercising parser + checker integration. Import-only checker smoke passed quickly, while parser + checker timed out even on detached HEAD `41c89210`. The checker boundary is `SelfhostModuleAst -> SelfhostModuleCheckSummary`; parser integration should be tested by integration tests, not by every focused checker doctest.

## 検証

- `node nodesrc/test_selfhost_module_checker_split_contract.js`: passed
- `node nodesrc/test_selfhost_proof_entry_contract.js`: passed
- `node nodesrc/test_selfhost_checker_report_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl -i stdlib/neplg2/core/check/checker.nepl --no-tree --dist web/dist -o tmp/agent1-selfhost-checker-doctest-timeout.json -j 1 --assert-io`: 2/2 passed under the default timeout.
