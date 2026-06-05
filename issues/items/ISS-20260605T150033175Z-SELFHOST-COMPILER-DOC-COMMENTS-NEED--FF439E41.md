---
id: ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41
title: "selfhost compiler doc comments need Zenn-policy section coverage"
area: selfhost
status: open
resolved: false
priority: P1
type: doc
created: 2026-06-05
updated: 2026-06-06
target: "stdlib/neplg2/**, nodesrc/test_selfhost_documentation_contract.js"
---

# ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41: selfhost compiler doc comments need Zenn-policy section coverage

## 概要

stdlib/neplg2 currently has remaining documentation gaps after the selfhost documentation contract baseline: moduleNoDoc=77, moduleNoDoctest=60, declarationNoDoc=304, declarationNoDoctest=1434, publicNoDoc=51, publicNoDoctest=1239, privateNoDoc=253, privateNoDoctest=195. The module doc count uses an explicit `//: # ...` heading at the file front rather than treating the first function doc as a module doc. A baseline-only gate prevents no-doc increases but does not by itself prove that each fixed declaration explains purpose, contract, return/error cases, complexity, and examples as required by the Zenn policy.

This baseline is not an accepted quality level. It is a fail-closed debt boundary for missing module/declaration comments: the no-doc counters must not increase, every newly fixed slice must receive section-level checks, and the remaining gaps stay open in this issue until they are either fixed or split into narrower root-cause issues. The no-doctest counters remain visible report-only debt because adding a careful doc comment to a previously undocumented declaration can temporarily increase the "doc exists but doctest is absent" count. The gate must not use file count, declaration count, line count, doc-comment length limits, or no-doctest count increases as a substitute for checking module boundaries and documentation contracts.

## 対象

- `stdlib/neplg2/**, nodesrc/test_selfhost_documentation_contract.js`

## 根拠

- Zenn 記事 `https://zenn.dev/bem130/articles/1b352797de94e7` は、ドキュメントコメントに目的、使用目的、計算量、典型例、実装者が守る contract、`Option` / `Result` など enum 戻り値の条件分岐、契約と現状実装の分離を記述する方針を定めている。
- 2026-06-05 の selfhost Zenn review gate hardening で `nodesrc/test_selfhost_documentation_contract.js` を追加し、`stdlib/neplg2` の documentation baseline と一部 public declaration の section coverage を検査し始めた。
- 同 review の subagent 指摘では、baseline-only gate は doc gap 増加を防げるが、未整備の public declaration が Zenn 方針を満たしたとは言えないため、残件を issue / note に固定して段階的に fail-closed 範囲を広げる必要があるとされた。
- 2026-06-06 の再レビューでは、file count / declaration count 下限は正当な削除や分割を妨げる size-ish gate になり得るため撤廃し、残gapをこの issue で明示的に追跡することを検査条件にした。

## 問題

stdlib/neplg2 currently has remaining documentation gaps after the selfhost documentation contract baseline: moduleNoDoc=77, moduleNoDoctest=60, declarationNoDoc=304, declarationNoDoctest=1434, publicNoDoc=51, publicNoDoctest=1239, privateNoDoc=253, privateNoDoctest=195. The module doc count uses an explicit `//: # ...` heading at the file front rather than treating the first function doc as a module doc. A baseline-only gate prevents no-doc increases but does not by itself prove that each public declaration explains purpose, contract, return/error cases, complexity, and examples as required by the Zenn policy. A no-doctest counter can increase when a declaration moves from "no doc" to "doc without runnable example", so that counter is tracked as visible debt rather than used to block documentation growth.

## 影響

Selfhost compiler implementation can appear source-policy clean while important compiler contracts, Result/Option branches, enum diagnostic conditions, ownership boundaries, and complexity guarantees remain undocumented or underdocumented. This weakens subagent review and makes later selfhost implementation work easier to regress.

## 修正方針

Expand nodesrc/test_selfhost_documentation_contract.js slice by slice. For each touched stdlib/neplg2 module, require public declarations and high-risk private helper boundaries to carry the relevant Zenn-policy sections such as purpose, contract, return/error cases, complexity, and doctest/report examples where the API is stable enough for runnable examples. Keep no-doc baseline counts decreasing and record accepted remaining gaps until they reach zero or are split into narrower issues. Keep no-doctest counters visible, but do not let them block adding careful comments to previously undocumented declarations.

Do not treat the raw number of files, declarations, lines, or doc comment lines as a quality proxy. A refactor that removes a public helper or folds a private helper into a clearer authority boundary should be judged by the resulting documentation contract and source-policy coverage, not by size preservation.

The 2026-06-06 correction expands the fixed slice to `stdlib/neplg2/core/check/expr/ascription.nepl`, requiring public ascription projection APIs to document purpose, owner contract, return/error conditions, and complexity.

The later 2026-06-06 correction expands the fixed slice to `stdlib/neplg2/core/check/expr/argument.nepl` and `stdlib/neplg2/core/check/expr/call_reduce.nepl`. It requires typed error authorities, owner-returning payloads, source-backed argument reducers, nested direct call reducers, and call reduction entries to document purpose, owner/evidence contract, Result/enum return conditions, and complexity. It also changes the documentation contract test so no-doctest counters are report-only debt, preserving the instruction that checks must not discourage adding detailed comments.

## 検証

node nodesrc/test_selfhost_documentation_contract.js; node nodesrc/test_selfhost_zenn_review_gate_contract.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues
