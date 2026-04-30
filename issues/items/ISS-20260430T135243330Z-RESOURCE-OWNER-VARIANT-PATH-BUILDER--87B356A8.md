---
id: ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8
title: "Resource owner variant path builder exceeds responsibility split policy"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8: Resource owner variant path builder exceeds responsibility split policy

## 概要

Remote main expanded owner_summary_variant_paths.rs to 637 lines while source policy limit is 380. Result owner variant path enumeration, condition propagation, call effect reservation, and returned owner path collection are concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

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
