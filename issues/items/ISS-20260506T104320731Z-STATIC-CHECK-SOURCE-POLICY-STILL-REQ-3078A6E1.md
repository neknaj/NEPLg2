---
id: ISS-20260506T104320731Z-STATIC-CHECK-SOURCE-POLICY-STILL-REQ-3078A6E1
title: "static check source policy still requires removed legacy move_check pass"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nodesrc/test_static_check_boundary_responsibility.js, nepl-core/src/passes/mod.rs"
---

# ISS-20260506T104320731Z-STATIC-CHECK-SOURCE-POLICY-STILL-REQ-3078A6E1: static check source policy still requires removed legacy move_check pass

## 概要

Remote main removed the legacy move_check pass, but nodesrc/test_static_check_boundary_responsibility.js still asserts passes/move_check.rs exists. node nodesrc/run_source_policy_regressions.js --warn-only now reports this stale policy as a warning.

## 対象

- `nodesrc/test_static_check_boundary_responsibility.js, nepl-core/src/passes/mod.rs`

## 根拠

- `origin/main` を `f30a5627` まで fast-forward した後、legacy `nepl-core/src/passes/move_check.rs` と `nepl-core/src/passes/move_check/**` は削除済みになっている。
- その一方で `nodesrc/test_static_check_boundary_responsibility.js` は `passes/move_check.rs` の存在を `assertFile` で要求し続けている。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は `missing passes/move_check.rs` を warning として報告した。これは stdlib math 分割とは独立した remote main 由来の stale policy である。

## 問題

Remote main removed the legacy move_check pass, but nodesrc/test_static_check_boundary_responsibility.js still asserts passes/move_check.rs exists. node nodesrc/run_source_policy_regressions.js --warn-only now reports this stale policy as a warning.

## 影響

Source policy regressions no longer accurately reflect the post-ResourceIR static-check architecture. Leaving the stale assertion hides real responsibility-boundary regressions behind an expected failure and weakens CI signal.

## 修正方針

Update the static check responsibility policy to track the current ResourceIR/static-check modules and explicitly reject reintroducing the removed legacy move_check pass instead of requiring it.

## 検証

Run node nodesrc/test_static_check_boundary_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only; both must pass without stale move_check warnings.
