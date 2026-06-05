---
id: ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41
title: "selfhost compiler doc comments need Zenn-policy section coverage"
area: selfhost
status: open
resolved: false
priority: P1
type: doc
created: 2026-06-05
updated: 2026-06-05
target: "stdlib/neplg2/**, nodesrc/test_selfhost_documentation_contract.js"
---

# ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41: selfhost compiler doc comments need Zenn-policy section coverage

## 概要

stdlib/neplg2 currently has remaining documentation gaps after the selfhost documentation contract baseline: moduleNoDoc=77, moduleNoDoctest=60, declarationNoDoc=304, declarationNoDoctest=1434, publicNoDoc=51, publicNoDoctest=1239, privateNoDoc=253, privateNoDoctest=195. The module doc count uses an explicit `//: # ...` heading at the file front rather than treating the first function doc as a module doc. A baseline-only gate prevents increases but does not by itself prove that each public declaration explains purpose, contract, return/error cases, complexity, and examples as required by the Zenn policy.

## 対象

- `stdlib/neplg2/**, nodesrc/test_selfhost_documentation_contract.js`

## 根拠

- Zenn 記事 `https://zenn.dev/bem130/articles/1b352797de94e7` は、ドキュメントコメントに目的、使用目的、計算量、典型例、実装者が守る contract、`Option` / `Result` など enum 戻り値の条件分岐、契約と現状実装の分離を記述する方針を定めている。
- 2026-06-05 の selfhost Zenn review gate hardening で `nodesrc/test_selfhost_documentation_contract.js` を追加し、`stdlib/neplg2` の documentation baseline と一部 public declaration の section coverage を検査し始めた。
- 同 review の subagent 指摘では、baseline-only gate は doc gap 増加を防げるが、未整備の public declaration が Zenn 方針を満たしたとは言えないため、残件を issue / note に固定して段階的に fail-closed 範囲を広げる必要があるとされた。

## 問題

stdlib/neplg2 currently has remaining documentation gaps after the selfhost documentation contract baseline: moduleNoDoc=77, moduleNoDoctest=60, declarationNoDoc=304, declarationNoDoctest=1434, publicNoDoc=51, publicNoDoctest=1239, privateNoDoc=253, privateNoDoctest=195. The module doc count uses an explicit `//: # ...` heading at the file front rather than treating the first function doc as a module doc. A baseline-only gate prevents increases but does not by itself prove that each public declaration explains purpose, contract, return/error cases, complexity, and examples as required by the Zenn policy.

## 影響

Selfhost compiler implementation can appear source-policy clean while important compiler contracts, Result/Option branches, enum diagnostic conditions, ownership boundaries, and complexity guarantees remain undocumented or underdocumented. This weakens subagent review and makes later selfhost implementation work easier to regress.

## 修正方針

Expand nodesrc/test_selfhost_documentation_contract.js slice by slice. For each touched stdlib/neplg2 module, require public declarations to carry the relevant Zenn-policy sections such as purpose, contract, return/error cases, complexity, and doctest/report examples. Keep baseline counts decreasing and record accepted remaining gaps until they reach zero or are split into narrower issues.

## 検証

node nodesrc/test_selfhost_documentation_contract.js; node nodesrc/test_selfhost_zenn_review_gate_contract.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues
