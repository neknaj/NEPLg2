---
id: ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8
title: "Resource owner variant path builder exceeds responsibility split policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-05-06
target: "nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8: Resource owner variant path builder exceeds responsibility split policy

## 概要

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `owner_summary_variant_paths.rs` は 637 lines まで肥大化し、variant return path traversal、branch/match condition propagation、variant construction inspection、match arm entry state mutation、returned owner projection collection を同じ module に保持していた。
- Stage 4 Resource check は Resource IR owner obligation の根拠を監査可能に保つ必要があるため、path traversal の orchestration と condition/construct/match-entry の意味論を同じ file に混在させると、後続の MemPtr/non-owning pointer と OwnedRegion/storage owner の修正が再び巨大 checker 化する。
- `nodesrc/test_resource_checker_responsibility.js` は `owner_summary_variant_paths.rs` に 380 lines 上限を設けており、分割前はこの上限を直接超えていた。

## 問題

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 影響

Resource IR owner summary logic is drifting back toward a monolithic checker. A local direct responsibility check reports the split-policy violation. GitHub Actions run `25157230630` for `f108cebd` still passed the aggregate `Source policy regressions` step, so this issue is not recorded as the confirmed Actions failure root cause. It remains a P1 design issue because selfhost static-check design becomes harder to copy safely if owner variant path enumeration, condition refinement, reservation, and returned path collection stay concentrated in one module.

## 修正方針

Split owner variant path logic into smaller modules such as path collection, condition refinement, reserved effect handling, and path application. Keep owner_summary_variant_paths.rs as orchestration only and update the source policy limits after the split.

## 検証

For review status, confirm GitHub Actions with `gh run view <run-id> --json ...` and distinguish Actions results from local direct checks. For implementation work, run the direct responsibility policy and Resource IR regression after splitting:

- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/run_source_policy_regressions.js`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`

## 2026-04-30 review correction

GitHub Actions run `25157230630` for `f108cebd` passed the `build` job's `Source policy regressions` step. The responsibility problem was found by local direct policy confirmation during review work, not by a failing Actions source-policy step. At that point the issue remained open because the module was still over-concentrated for Resource IR owner summary design.

## 2026-05-06 対応結果

`owner_summary_variant_paths.rs` を path traversal orchestration に戻し、混在していた責務を次の module へ分割した。

- `owner_summary_variant_construct.rs`: return value に対応する enum construct と payload suffix の抽出。
- `owner_summary_variant_conditions.rs`: branch condition と payload condition を owner variant summary 用の condition fact に変換。
- `owner_summary_variant_match.rs`: match arm entry 時の payload owner transfer、raw alias/view、pending realloc、variant owner effect の適用。

分割後の行数は `owner_summary_variant_paths.rs` 337 lines、`owner_summary_variant_conditions.rs` 182 lines、`owner_summary_variant_construct.rs` 61 lines、`owner_summary_variant_match.rs` 90 lines で、対象 module は責務分割ポリシー上限内に収まった。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在確認と line limit を追加し、owner variant path builder の再肥大化を検出できるようにした。

検証:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_owner_into_constructed_aggregate -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_aggregate_owner_projection -- --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: owner variant module の分割は通過。別件として `lower.rs has 1315 lines; responsibility split limit is 1300` を検出したため、`ISS-20260505T184012396Z-RESOURCE-IR-LOWERING-TRAVERSAL-EXCEE-8A0A5A86` を追加した。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: source-based tests は既存の `ShadowSameSignatureCallable` warning を `typecheck_resource_source` helper が失敗扱いするため一部未完了。今回の分割後の直接 Resource IR owner regression は通過した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
