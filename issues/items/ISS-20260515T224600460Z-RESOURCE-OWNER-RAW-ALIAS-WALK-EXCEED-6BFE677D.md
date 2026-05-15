---
id: ISS-20260515T224600460Z-RESOURCE-OWNER-RAW-ALIAS-WALK-EXCEED-6BFE677D
title: "resource owner raw alias walk exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "nepl-core/src/resource/owner_summary_raw_alias_walk.rs, nepl-core/src/resource/owner_summary_raw_alias_branch.rs, nodesrc/test_resource_checker_responsibility.js"
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

## 2026-05-16 修正

raw owner alias walker から branch path merge を分離した。

- `owner_summary_raw_alias_walk.rs` は `ResourceOp` dispatch、linear transfer、loop/match/direct call の alias propagation に戻した。
- branch の then/else path clone、path-local alias collection、output raw view merge は `owner_summary_raw_alias_branch.rs` へ移した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、80 行上限を追加し、branch merge 責務が walker へ戻った場合に検出できるようにした。

検証:

- `cargo test -p nepl-core owner_summary_raw_transfer -- --nocapture`: 4 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_accepts_vec_owner_result_return_identity -- --nocapture`: 1 passed
- `node nodesrc/test_resource_checker_responsibility.js`: `owner_summary_raw_alias_walk.rs` blocker は解消。次の別 issue として `owner_summary_raw_use_call.rs has 136 lines; responsibility split limit is 90` を検出したため `ISS-20260515T225118666Z-RESOURCE-OWNER-RAW-USE-CALL-SUMMARY--3E21FFD7` に分離した。
