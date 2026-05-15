---
id: ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F
title: "Resource owner_variant module exceeds responsibility split limit again"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/owner_variant.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F: Resource owner_variant module exceeds responsibility split limit again

## 概要

After splitting owner_return_apply extent helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_variant.rs has 871 lines while the enforced limit is 840. Variant match/materialization application has grown again after the previous lifecycle and record splits.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_variant.rs has 871 lines; responsibility split limit is 840` で失敗した。
- `owner_variant.rs` には match/materialization orchestration に加え、pending variant owner consumption / return の実適用、reserved owner state、variant extent requirement merge helper が戻っていた。
- pending owner application helper は enum payload owner transfer の memory-safety authority だが、state orchestration 本体とは分けて監査できる責務である。

## 問題

After splitting owner_return_apply extent helpers, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_variant.rs has 871 lines while the enforced limit is 840. Variant match/materialization application has grown again after the previous lifecycle and record splits.

## 影響

owner_variant.rs is a memory-safety authority for enum payload owner transfer and pending variant owner effects. Letting it grow past the policy limit makes it harder to audit exhaustive state transitions and increases the chance of mixing lifecycle, condition, and materialization rules again.

## 修正方針

Audit owner_variant.rs for the newly accumulated responsibility, split a coherent part into a focused module without changing semantics, lower/keep policy limits so future growth is caught, and preserve ResourceIR owner variant regressions.

## 検証

Run cargo fmt -p nepl-core --check, focused owner variant ResourceIR tests, nodesrc/test_resource_checker_responsibility.js, source policy warn-only, issues check, and diff whitespace check.

## 対応

`owner_variant.rs` から pending variant owner application helper を `owner_variant_apply.rs` へ分離した。

- `pending_consumption_source` / `consume_pending_variant_owner` / `pending_return_source` / `apply_pending_variant_owner_return` を移動し、match/materialization orchestration から実際の owner transfer detail を切り離した。
- reserved owner state helper と variant extent requirement merge helper も同 module に移し、pending application に必要な補助責務をまとめた。
- `owner_variant.rs` は pending effect model、match arm 適用、result materialization、summary collection orchestration に集中した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_variant_apply.rs` を mandatory module / line-limit 監視として追加した。
- 分割後の行数は `owner_variant.rs=677`、`owner_variant_apply.rs=215`。

## 検証結果

- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir owner_check_preserves_branch_result -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir owner_check_prefers_live_return_owner -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir owner_check_reports_leaked_conditional_owner_return -- --nocapture`: 1 passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: Resource checker は pass。残 warning は documentation contract `ISS-20260515T130627053Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-3CDEFF1A`。
