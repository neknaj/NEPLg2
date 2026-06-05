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

The 2026-06-06 ascription correction expands the fixed slice again to the private ascription range/projection helpers, `block_body.nepl`, and `body_line.nepl`. It removes an unused ascription error wrapper instead of documenting dead code, documents token-range authority, owner cleanup, `MissingExpressionTail` / `InvalidExpressionRange` / `TypeProjectFailed` branches, and adds fixed `doctest` section checks only to representative public ascription projection entries. A follow-up review found that a bare `neplg2:test` marker is not enough, so the contract now also requires the ascription projection doctests to call the target projection API and its accessor/free boundary directly. The projection docs also state which helper closes the arena owner for tail/head range failure versus type projection failure. This keeps the Zenn requirement for examples and executable doc tests without reintroducing broad no-doctest gates that would discourage adding careful comments.

The later 2026-06-06 stage0 correction expands the fixed slice to `stdlib/neplg2/core/check/expr/stage0.nepl`. It requires the call reduction smoke fixture helpers to document purpose, fixture-vs-production responsibility, owner cleanup, typed enum error conditions, partial-application rejection, expected-result mismatch, generic evidence missing, overload ambiguity, and complexity. The documentation contract adds targeted section requirements for these declarations only, without adding any line-count, file-count, declaration-count, or doc-comment-length gate.

The stage0 review found that the first pass still missed typed failure boundary helpers and the public smoke API from fixed section checks, and that function type allocation failure cleanup was described too broadly. The corrected slice now gates the argument type mismatch fixture, ascribed argument unsupported fixture, their prefix fixture builders, and `selfhost_check_expr_stage0` itself. It also documents that parameter Vec failures are cleaned by the stage0 helper, while `selfhost_type_arena_add_function` consumes and closes the arena/parameter owners on function type allocation failure.

The later 2026-06-06 stage1 correction expands the fixed slice to the stage1 value context and function value argument fixture boundary in `stdlib/neplg2/core/check/expr/stage1.nepl`. It requires the value context constructor/accessors/free path, binding-only / typed-value / function context fixtures, candidate Vec wrappers, one-argument function type fixture, function-value consumer type fixture, and `takes @add` / `takes add` segment/token fixtures to document purpose, owner/borrow contracts, Result branches, explicit `@` function value semantics, no-partial-application rejection, and complexity. Remaining stage1 reducer/run/body-line smoke gaps stay open for a later slice.

The 2026-06-06 Zenn review continuity correction connects the review process itself to a fail-closed source policy. `nodesrc/test_selfhost_zenn_review_gate_contract.js` now checks the latest selfhost checkpoint in `note.n.md`, and `nodesrc/selfhost_zenn_review_response_check.js --record <note-or-issue.md>` can reject a review response whose summary and decision were not recorded in `note.n.md` or an issue. This prevents subagent review from remaining only in a chat transcript or temporary file.

## 完了条件

- `moduleNoDoc`, `declarationNoDoc`, `publicNoDoc`, and `privateNoDoc` for `stdlib/neplg2/**` reach zero, or each remaining root cause is split into a narrower open issue with an owner boundary, impact, completion conditions, and verification plan.
- Every fixed selfhost slice has targeted section requirements for purpose, contract, return/error cases, complexity, and representative doctest/report examples where the public API is stable enough.
- No gate uses file count, declaration count, line count, byte count, file size, doc comment length, comment line count, or no-doctest count increases as a reason to suppress detailed comments.
- The latest selfhost checkpoint in `note.n.md` records Zenn re-check, `AGENTS.md` check, `policy/spec` and `implementation/test` review axes, subagent review outcome, `classification` / `decision` / `source_policy` / `verify`, executed and unexecuted verification, existing warnings, warnings introduced by the current diff, and next-slice residual work.
- Final acceptance of a selfhost slice validates the subagent response with `nodesrc/selfhost_zenn_review_response_check.js --record <note-or-issue.md>` so the response is tied to durable note or issue evidence.

## review 証跡

- 2026-06-05: initial selfhost Zenn review gate hardening confirmed the baseline was not a quality acceptance line and added the open issue boundary.
- 2026-06-06: ascription, argument/call-reduce, stage0, and stage1 slices used subagent review to identify missing section requirements and owner-cleanup ambiguities, then fixed Blocker findings in the same slice.
- 2026-06-06: review continuity follow-up found that packet/response checks were not enough unless the accepted response was also recorded. The response checker and note checkpoint contract now cover this persistence requirement.
- Any future Blocker that cannot be fixed in the same branch must be added to this issue or split into a narrower issue with root cause, impact, completion conditions, fail-closed boundary, and verification plan before merge.

## 検証

node nodesrc/test_selfhost_documentation_contract.js; node nodesrc/test_selfhost_zenn_review_gate_contract.js; node nodesrc/run_source_policy_regressions.js --warn-only; node nodesrc/issues.js check --dir issues
