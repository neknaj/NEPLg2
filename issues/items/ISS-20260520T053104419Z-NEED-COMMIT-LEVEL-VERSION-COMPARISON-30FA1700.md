---
id: ISS-20260520T053104419Z-NEED-COMMIT-LEVEL-VERSION-COMPARISON-30FA1700
title: "Need commit-level version comparison for tests performance and repo metrics"
area: tooling
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/compare_git_versions.js, repo_metrics.ts, doc/neplg2/version_comparison_metrics_plan.md"
---

# ISS-20260520T053104419Z-NEED-COMMIT-LEVEL-VERSION-COMPARISON-30FA1700: Need commit-level version comparison for tests performance and repo metrics

## 概要

There is no integrated system for selecting git commits and comparing doctest pass rate, compile time, runtime, execution duration, and repo_metrics.ts code-size metrics under the same inputs.

## 対象

- `nodesrc/compare_git_versions.js, repo_metrics.ts, doc/neplg2/version_comparison_metrics_plan.md`

## 根拠

- `nodesrc/tests.js` は doctest 結果 JSON に `summary` と result-level `timing.compile_ms` / `timing.run_ms` / `duration_ms` を出力している。
- `repo_metrics.ts` は repository の files / lines / bytes / source / doc_comment / document / test / testCases を JSON 出力できる。
- しかし、これらを git commit 単位で同じ条件に揃えて比較する tool がなく、過去版との性能・規模・通過率の比較が手作業になっている。
- 静的検査大規模修正では一時的なテスト悪化やコンパイル時間変化を慎重に評価する必要があるため、commit 単位の比較結果を JSON と Markdown で残せる仕組みが必要である。

## 問題

There is no integrated system for selecting git commits and comparing doctest pass rate, compile time, runtime, execution duration, and repo_metrics.ts code-size metrics under the same inputs.

## 影響

Performance regressions, test pass-rate changes, and repository scale changes must be checked manually across tools, making long-running compiler work difficult to evaluate consistently.

## 修正方針

Add a worktree-based comparison tool that accepts multiple git refs, runs repo_metrics.ts and focused doctests per ref, emits structured JSON and Markdown comparison tables, and supports accurate per-ref dist builds when requested.

設計書: [NEPLg2 git commit version comparison plan](../../doc/neplg2/version_comparison_metrics_plan.md)

`nodesrc/compare_git_versions.js` を追加し、`--rev` で指定した各 ref に対して一時 `git worktree` を作成する。測定ロジックは current checkout の comparison tool / `repo_metrics.ts` / `nodesrc/tests.js` を使い、対象 source は worktree から読む。

コンパイラ自体の速度を比較する場合は `--build-cmd` と `--dist-rel` で各 commit の compiler artifact を作る。既存 dist を使った軽量比較では `--dist-current` を使えるが、その場合は compiler binary の速度比較ではなく、source/test/stdlib 入力差分の比較として扱う。

## 検証

Add focused node tests for summary/delta calculation and run the tool on two refs with metrics-only or a tiny doctest input.

## 2026-05-20 実装結果

`nodesrc/compare_git_versions.js` を追加し、`--rev` で指定した git ref ごとに一時 worktree を作成して比較できるようにした。各 revision では `repo_metrics.ts` を対象 worktree root に対して実行し、必要に応じて `nodesrc/tests.js` も対象 worktree cwd で実行する。

出力は `neplg2-git-version-comparison/v1` の JSON と Markdown table である。JSON には revision ごとの commit、test summary、compile/run/duration timing 集計、repo metrics totals / by_area / by_content_kind / by_extension、実行 command の status と出力末尾を含める。Markdown は Discord / issue に貼るための一覧表で、詳細確認の正本は JSON とする。

`--build-cmd` と `--dist-rel` を使うと commit ごとの compiler artifact を作って比較できる。`--dist-current` は軽量比較用として残し、compiler binary 自体の速度比較には使わないことを doc に明記した。

検証:

- `node nodesrc/test_compare_git_versions_summary.js`: pass
- `node nodesrc/compare_git_versions.js --rev HEAD --metrics-only -o tmp/agent1-version-compare-smoke.json --markdown tmp/agent1-version-compare-smoke.md --command-timeout-ms 300000`: pass
- `node nodesrc/compare_git_versions.js --rev HEAD -i tests/compiler/impl_visibility.n.md --dist-current web/dist --no-tree -o tmp/agent1-version-compare-test-smoke.json --markdown tmp/agent1-version-compare-test-smoke.md --command-timeout-ms 300000`: total=1, passed=1
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
