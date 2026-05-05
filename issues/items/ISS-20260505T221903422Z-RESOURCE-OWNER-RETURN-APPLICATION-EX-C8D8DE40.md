---
id: ISS-20260505T221903422Z-RESOURCE-OWNER-RETURN-APPLICATION-EX-C8D8DE40
title: "Resource owner return application exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_view.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T221903422Z-RESOURCE-OWNER-RETURN-APPLICATION-EX-C8D8DE40: Resource owner return application exceeds responsibility split limit

## 概要

owner_return.rs has grown to 433 lines while the Resource checker responsibility policy limits it to 400. It now mixes direct and indirect call owner-return orchestration, unknown callback fallback, owner-return summary application, parameter consumption, and non-owning raw-address-view classification.

## 対象

- `nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_view.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_return.rs has 433 lines; responsibility split limit is 400` で停止した。
- `owner_return.rs` は direct call / known indirect call / unknown callback fallback に加え、`OwnerReturnSummary` 適用、projection owner return materialization、consumed parameter owner 処理、non-owning raw address view 判定まで同居していた。
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)

## 問題

owner_return.rs has grown to 433 lines while the Resource checker responsibility policy limits it to 400. It now mixes direct and indirect call owner-return orchestration, unknown callback fallback, owner-return summary application, parameter consumption, and non-owning raw-address-view classification.

## 影響

Resource IR owner-return logic is re-concentrating across MemPtr/OwnedRegion and raw view boundaries. This weakens auditability of memory-safety checks and makes it easier for future Resource IR changes to bypass the intended typed owner/checker separation.

## 修正方針

Split owner-return handling by responsibility: keep owner_return.rs focused on call orchestration and unknown callback fallback, move owner-return summary application and parameter consumption into a dedicated module, and move non-owning raw-address-view classification into a dedicated typed helper module. Update the responsibility regression to require the split modules and prevent re-growth.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only; focused nepl-core resource_ir owner-return tests; cargo check -p nepl-core --tests; node nodesrc/issues.js check; git diff --check

## 2026-05-06 対応結果

`owner_return.rs` を call orchestration に戻し、owner return summary application と raw-view classification を別 module へ分離した。

- `owner_return.rs`: direct call / indirect call / unknown callback fallback の owner return orchestration を担当する。
- `owner_return_apply.rs`: `OwnerReturnSummary` / `OwnerProjectionReturnSummary` の caller-side application、returned owner transfer、fresh/maybe owner materialization、consumed parameter owner の move-out を担当する。
- `owner_return_view.rs`: non-owning raw address view の判定を担当し、raw `i32` view を owner transfer 対象として誤消費しない境界を独立させた。
- `nodesrc/test_resource_checker_responsibility.js`: 新 module の存在、`ResourceOwnerCheckEngine` 境界、行数上限を固定した。`owner_return.rs` 上限は 400 から 220 に下げ、再集中を検出しやすくした。

検証:

- `cargo fmt -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: 6 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: downstream policy は継続実行。`owner_return.rs` 超過は解消し、次の別件として `coverage_hir.rs has 463 lines; responsibility split limit is 420` を検出したため、`ISS-20260505T222215631Z-RESOURCE-HIR-COVERAGE-CHECKER-EXCEED-BACF550C` を追加した。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
