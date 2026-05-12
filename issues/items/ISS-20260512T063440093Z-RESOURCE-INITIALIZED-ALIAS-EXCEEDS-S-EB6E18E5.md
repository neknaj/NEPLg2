---
id: ISS-20260512T063440093Z-RESOURCE-INITIALIZED-ALIAS-EXCEEDS-S-EB6E18E5
title: "Resource initialized alias exceeds split limit after i32 fact changes"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T063440093Z-RESOURCE-INITIALIZED-ALIAS-EXCEEDS-S-EB6E18E5: Resource initialized alias exceeds split limit after i32 fact changes

## 概要

After splitting owner summary variant conditions, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: initialized_alias.rs has 524 lines while the responsibility split limit is 520. The alias tracking module has regrown slightly past its policy boundary.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting owner summary variant conditions, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: initialized_alias.rs has 524 lines while the responsibility split limit is 520. The alias tracking module has regrown slightly past its policy boundary.

## 影響

Initialized alias tracking is a core part of memory-safety analysis. Letting it grow past the responsibility boundary makes alias facts and i32 condition tracking harder to audit.

## 修正方針

Do not raise the limit. Split small helper logic or recently added i32/condition alias handling out of initialized_alias.rs into an existing focused module or a new helper module, then update source policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.
