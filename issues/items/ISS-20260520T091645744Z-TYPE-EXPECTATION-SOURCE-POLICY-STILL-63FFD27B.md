---
id: ISS-20260520T091645744Z-TYPE-EXPECTATION-SOURCE-POLICY-STILL-63FFD27B
title: "type expectation source policy still assumes monolithic overload selection"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/test_type_expectation_model_policy.js, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/overload_candidate.rs, nepl-core/src/typecheck/overload_narrowing.rs"
---

# ISS-20260520T091645744Z-TYPE-EXPECTATION-SOURCE-POLICY-STILL-63FFD27B: type expectation source policy still assumes monolithic overload selection

## 概要

After overload candidate and narrowing responsibilities were split out of overload_selection.rs, nodesrc/test_type_expectation_model_policy.js still searches overload_selection.rs for the candidate rejection enums, materialization phase model, narrowing stage model, and direct one-way declared-result pruning spelling.

## 対象

- `nodesrc/test_type_expectation_model_policy.js, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/overload_candidate.rs, nepl-core/src/typecheck/overload_narrowing.rs`

## 根拠

- `node nodesrc/test_type_expectation_model_policy.js` が `overload selection must use declared result shape before candidate instantiation when expected result is available` で失敗していた。
- `overload_selection.rs` は現在、declared result expectation pruning を `result_may_satisfy_expectation` helper に持ち、candidate model / rejection stats / materialization phase / ambiguity payload は `overload_candidate.rs`、narrowing algorithm は `overload_narrowing.rs` に分割済みである。
- policy が旧 monolithic file と旧 one-way helper spelling を要求すると、実装の正しさではなく source policy の古さで CI が警告を出す。

## 問題

After overload candidate and narrowing responsibilities were split out of overload_selection.rs, nodesrc/test_type_expectation_model_policy.js still searches overload_selection.rs for the candidate rejection enums, materialization phase model, narrowing stage model, and direct one-way declared-result pruning spelling.

## 影響

The source policy now fails even when the implementation is correct, and it no longer monitors the typed overload model at the actual module boundaries introduced by the responsibility split.

## 修正方針

Update the policy to inspect overload_selection.rs for candidate construction and pre-instantiation expectation pruning, overload_candidate.rs for typed candidate/rejection/materialization payloads, and overload_narrowing.rs for ambiguity narrowing construction.

## 検証

Run node nodesrc/test_type_expectation_model_policy.js, node nodesrc/issues.js check, and git diff --check.

## 2026-05-20 Agent 1 修正

`nodesrc/test_type_expectation_model_policy.js` を responsibility split 後の構造へ追従させた。

- `overload_selection.rs` は candidate construction と pre-instantiation pruning を監視する。
- `result_may_satisfy_expectation` が declared result と expected result を双方向に照合し、unresolved expected variable を持つ候補を誤って落とさないことを監視する。
- `overload_candidate.rs` は `OverloadCandidateRejection`、materialization phase、stats、ambiguity payload を監視する。
- `overload_narrowing.rs` は `OverloadAmbiguityReason::after_stage(...)` の構築を監視する。

検証:

- `node nodesrc/test_type_expectation_model_policy.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
