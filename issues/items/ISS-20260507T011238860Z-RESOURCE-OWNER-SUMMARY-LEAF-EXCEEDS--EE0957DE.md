---
id: ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE
title: "Resource owner summary leaf exceeds responsibility split policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_summary_leaf.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE: Resource owner summary leaf exceeds responsibility split policy

## 概要

After splitting initialized availability checks, the Resource checker responsibility policy reveals owner_summary_leaf.rs at 387 lines against a 260-line limit, concentrating owner-summary leaf and record traversal logic.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260507T011054993Z-RESOURCE-INITIALIZED-AVAILABILITY-CH-ED41979E` の分割後、`node nodesrc/test_resource_checker_responsibility.js` が次の未解決超過として検出した。
- `owner_summary_leaf.rs has 387 lines; responsibility split limit is 260`
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ warning が残り、downstream CI は継続するが source policy debt として残っている。

## 問題

After splitting initialized availability checks, the Resource checker responsibility policy reveals owner_summary_leaf.rs at 387 lines against a 260-line limit, concentrating owner-summary leaf and record traversal logic.

## 影響

Owner obligation summaries can re-centralize in a large helper module, reducing match-based auditability of free-obligation transfer and making memory-safety regressions harder to localize.

## 修正方針

Split owner_summary_leaf.rs into coherent leaf classification and traversal/update modules without raising the policy limit, then register the new module in the responsibility policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split and focused owner-summary/resource tests.

## 2026-05-07 Agent 1 fixed

- `owner_summary_leaf.rs` から raw owner consumption の関数本体走査を `owner_summary_raw_consumption.rs` へ分離した。
- enum payload owner leaf 展開を `owner_summary_variant_leaf.rs` へ分離した。
- `owner_summary_leaf.rs` は型ごとの owner leaf projection と public entry に責務を絞った。
- source policy に新 module と上限を登録し、owner summary leaf 周辺の再集中を検出できるようにした。
- 行数は `owner_summary_leaf.rs` 236/260、`owner_summary_raw_consumption.rs` 122/140、`owner_summary_variant_leaf.rs` 42/80。
- 分割後の policy は次の未解決問題として `initialized_alias_flow.rs` 1034/550 行を露出したため、`ISS-20260507T011907998Z-RESOURCE-INITIALIZED-ALIAS-FLOW-EXCE-E65684BD` を追加した。

確認:

- `cargo fmt --check -p nepl-core`
- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core resource:: --lib`
- `trunk build`
- `node -e ... owner_summary_* line count`
- `node nodesrc/run_source_policy_regressions.js --warn-only`: owner summary leaf 側は解消。`initialized_alias_flow.rs` 超過のみ警告。
- `node nodesrc/issues.js check`
- `node nodesrc/tests.js -i tests/compiler/drop.n.md -i tests/compiler/drop_overwrite.n.md --no-tree --dist web/dist -o tmp/drop_agent1_after_owner_summary_leaf_split.json -j 1 --assert-io`
