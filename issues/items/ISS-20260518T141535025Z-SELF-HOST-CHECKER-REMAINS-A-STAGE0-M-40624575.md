---
id: ISS-20260518T141535025Z-SELF-HOST-CHECKER-REMAINS-A-STAGE0-M-40624575
title: "self-host checker remains a stage0 marker and does not validate module items"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/checker.nepl, stdlib/neplg2/core/pipeline.nepl, stdlib/neplg2/core/infra/diag.nepl, tests/stdlib/neplg2_checker.n.md"
---

# ISS-20260518T141535025Z-SELF-HOST-CHECKER-REMAINS-A-STAGE0-M-40624575: self-host checker remains a stage0 marker and does not validate module items

## 概要

self-host core/check/checker.nepl still exposes only a constant stage0 marker, so the pipeline can load a parsed module but cannot run even typed module-level validation. This hides later type/lifetime/effect checker work behind an unverified boundary.

## 対象

- `stdlib/neplg2/core/check/module.nepl, stdlib/neplg2/core/check/checker.nepl, stdlib/neplg2/core/pipeline.nepl, stdlib/neplg2/core/infra/diag.nepl, tests/stdlib/neplg2_checker.n.md`

## 根拠

- `stdlib/neplg2/core/check/checker.nepl` は stage0 marker だけを返しており、loaded root の AST を検査する public boundary がなかった。
- `stdlib/neplg2/core/pipeline.nepl` は root module load まではできるが、load 後に module item stream を checker へ渡す API を持っていなかった。
- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` では self-host 実装を Rust 側の flat file 構造へ戻さず、`core/check/` の最終階層へ実装本体を置くことを完了条件としている。

## 問題

self-host core/check/checker.nepl still exposes only a constant stage0 marker, so the pipeline can load a parsed module but cannot run even typed module-level validation. This hides later type/lifetime/effect checker work behind an unverified boundary.

## 影響

self-host progress appears larger than it is, and invalid or internally inconsistent module item streams cannot be rejected or summarized by a typed checker stage. Later static-check passes would have to depend on ad hoc caller-side assumptions.

## 修正方針

Add a final-hierarchy module checker in core/check/module.nepl that traverses SelfhostModuleItemKind via exhaustive match, expose checker.nepl as orchestration only, add checker-specific hierarchical diagnostics, and connect pipeline loaded roots to the checker boundary.

## 対応結果

- `stdlib/neplg2/core/check/module.nepl` を追加し、`SelfhostModuleItemKind` の exhaustive match で module item stream を検査・集計する `SelfhostModuleCheckSummary` を実装した。
- `checker.nepl` は public facade / orchestration に戻し、実装本体を巨大 root file に置かない構造にした。
- `SelfhostDiagnosticCode::Checker(SelfhostCheckerDiagnosticCode)` を追加し、checker diagnostic も階層 enum から stable code string を生成するようにした。
- `selfhost_pipeline_check_loaded_root` を追加し、VFS load 済み AST を checker boundary へ接続した。
- raw backend block の `#wasm:` / `#llvm-ir:` と raw text の対応を state machine で検査し、block 外 raw text と空 raw block を diagnostic にする回帰テストを追加した。
- checker diagnostic の primary label は Resource IR owner check が `SelfhostDiagnosticLabel` payload を誤分類するため一時的に使わず、根本問題は `ISS-20260518T142248720Z-RESOURCE-IR-OWNER-CHECK-MISCLASSIFIE-8A3C73E5` に分離した。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl -i stdlib/neplg2/core/check/checker.nepl -i stdlib/neplg2/core/pipeline.nepl -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-selfhost-checker-module.json -j 1 --dist web/dist --assert-io`: passed, total=5
- `node nodesrc/test_selfhost_checker_report_contract.js`: passed
- `node nodesrc/test_selfhost_diag_code_enum.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
