---
id: ISS-20260515T224600460Z-RESOURCE-OWNER-RAW-ALIAS-WALK-EXCEED-6BFE677D
title: "resource owner raw alias walk exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/owner_summary_raw_alias_walk.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T224600460Z-RESOURCE-OWNER-RAW-ALIAS-WALK-EXCEED-6BFE677D: resource owner raw alias walk exceeds responsibility split limit

## 概要

After owner extent check responsibility was split again, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_summary_raw_alias_walk.rs has 187 lines while the enforced limit is 180. Branch handling, match-arm alias propagation, raw view path merge, and direct call returned raw owner alias propagation are concentrated in one walker module.

## 対象

- `nepl-core/src/resource/owner_summary_raw_alias_walk.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_extent_check.rs` の責務分割後に `owner_summary_raw_alias_walk.rs has 187 lines; responsibility split limit is 180` を報告した。
- `owner_summary_raw_alias_walk.rs` は raw owner alias の全体走査に加え、branch path の別名 merge、match arm の別名伝搬、direct call return summary の raw owner alias 適用を同じ walker に持っている。
- 行数上限を緩めると Resource IR の raw owner view / alias proof が再び大型 module に集まり、静的検査のレビュー可能性を落とす。

## 問題

After owner extent check responsibility was split again, node nodesrc/test_resource_checker_responsibility.js reaches the next Resource IR responsibility blocker: owner_summary_raw_alias_walk.rs has 187 lines while the enforced limit is 180. Branch handling, match-arm alias propagation, raw view path merge, and direct call returned raw owner alias propagation are concentrated in one walker module.

## 影響

Static-check correctness work can continue to accumulate raw owner alias propagation complexity in one module. This weakens the policy that Resource IR proof code remains small enough for careful review and makes raw owner view soundness regressions harder to isolate.

## 修正方針

Split branch or match raw owner alias path propagation into a dedicated module without weakening line limits, then register the new module in resource/mod.rs and nodesrc/test_resource_checker_responsibility.js.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused nepl-core resource_ir raw owner alias tests, node nodesrc/issues.js check --dir issues, and git diff --check.
