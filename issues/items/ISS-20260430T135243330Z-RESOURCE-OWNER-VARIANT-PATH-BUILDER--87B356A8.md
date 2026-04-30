---
id: ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8
title: "Resource owner variant path builder exceeds responsibility split policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-05-01
target: "nepl-core/src/resource/owner_summary_variant_paths.rs, nepl-core/src/resource/owner_summary_variant_conditions.rs, nepl-core/src/resource/owner_summary_variant_construct.rs, nepl-core/src/resource/owner_summary_variant_entry.rs, nepl-core/src/resource/owner_summary_variant_unique.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8: Resource owner variant path builder exceeds responsibility split policy

## 概要

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `owner_summary_variant_paths.rs` は 681 行まで肥大化し、`nodesrc/test_resource_checker_responsibility.js` の 380 行上限を超えていた。
- 同 module に nested return path traversal、match arm entry state application、constructed variant reconstruction、condition refinement、unique collection helper が同居していた。
- Stage 4 Resource IR owner summary は self-host 側へ写す設計対象なので、variant path enumeration と condition refinement を一つの helper file に集中させると静的検査実装の保守性を落とす。

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

GitHub Actions run `25157230630` for `f108cebd` passed the `build` job's `Source policy regressions` step. The responsibility problem was found by local direct policy confirmation during review work, not by a failing Actions source-policy step. The issue remains open because the module is still over-concentrated for Resource IR owner summary design.

## 2026-05-01 対応結果

`owner_summary_variant_paths.rs` を path orchestration に戻し、次の責務を独立 module へ分離した。

- `owner_summary_variant_entry.rs`: match arm entry 時の owner/raw alias/function alias/pending realloc/variant effect state application。
- `owner_summary_variant_construct.rs`: returned value に対応する constructed enum variant と payload projection の復元。
- `owner_summary_variant_conditions.rs`: branch condition と payload condition の variant summary への変換。
- `owner_summary_variant_unique.rs`: variant summary list への重複排除 push helper。

分離後、`owner_summary_variant_paths.rs` は 356 行となり、issue 対象の 380 行上限を満たす。`nodesrc/test_resource_checker_responsibility.js` は新 module の存在と上限も監視する。

検証:

- `cargo fmt --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_safe_realloc_variant_return_preserves_err_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: variant path split policy is no longer reported; still warns on the separate `owner_check.rs has 906 lines; responsibility split limit is 800` regression.
- `node nodesrc/test_resource_checker_responsibility.js`: blocked by the separate `owner_check.rs` responsibility regression, not by `owner_summary_variant_paths.rs`.
