---
id: ISS-20260518T015305820Z-RESOURCE-OWNER-FLOW-RESPONSIBILITY-L-ED29F73C
title: "Resource owner checker responsibility limits fail after checker growth"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_release.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_apply_projection.rs, nepl-core/src/resource/owner_return_apply_source.rs, nepl-core/src/resource/owner_return_apply_place.rs, nepl-core/src/resource/raw_pointer_type.rs, nepl-core/src/resource/raw_pointer_type_tests.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260518T015305820Z-RESOURCE-OWNER-FLOW-RESPONSIBILITY-L-ED29F73C: Resource owner checker responsibility limits fail after checker growth

## 概要

Resource owner checker responsibility policy no longer passed after recent Resource IR owner and raw pointer summary growth. The first visible failure was `owner_flow.rs`, but after that split the same policy exposed `owner_return_apply.rs`, `owner_return_apply_source.rs`, and `raw_pointer_type.rs` as additional responsibility concentrations.

## 対象

- `nepl-core/src/resource/owner_flow.rs`
- `nepl-core/src/resource/owner_release.rs`
- `nepl-core/src/resource/owner_return_apply.rs`
- `nepl-core/src/resource/owner_return_apply_projection.rs`
- `nepl-core/src/resource/owner_return_apply_source.rs`
- `nepl-core/src/resource/owner_return_apply_place.rs`
- `nepl-core/src/resource/raw_pointer_type.rs`
- `nepl-core/src/resource/raw_pointer_type_tests.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` initially failed with `owner_flow.rs has 650 lines; responsibility split limit is 620`.
- After `owner_flow.rs` was split, the same policy exposed `owner_return_apply.rs has 427 lines; responsibility split limit is 410`.
- After return projection apply was split, the policy exposed `owner_return_apply_source.rs has 213 lines; responsibility split limit is 180`.
- After return-place helpers were split, the policy exposed `raw_pointer_type.rs has 138 lines; responsibility split limit is 120`.

## 問題

Resource owner checking had started to re-concentrate independent responsibilities:

- owner construction / transfer / release / availability were mixed in `owner_flow.rs`.
- root owner return application and projection-owner return application were mixed in `owner_return_apply.rs`.
- parameter consumption, copy-view predicates, and projection place construction were mixed in `owner_return_apply_source.rs`.
- raw pointer alias carrier semantics and regression tests were mixed in `raw_pointer_type.rs`.

## 影響

Resource IR owner checking becomes harder to audit and regressions in memory-safety state transitions become easier to hide because unrelated owner operations share large implementation files. The responsibility policy also becomes less useful if newly added semantic proof helpers and their regression tests stay in the same module until the policy limit is raised instead of preserving the boundary.

## 修正方針

Split by semantic responsibility while preserving existing owner-state semantics and keeping the responsibility policy unchanged:

- Move owner release / availability checks into `owner_release.rs`.
- Move projection-owner return application into `owner_return_apply_projection.rs`.
- Move return projection place construction and copy-owner-view predicate into `owner_return_apply_place.rs`.
- Move raw pointer carrier regression tests into `raw_pointer_type_tests.rs`.
- Register each new module in `mod.rs` and in `nodesrc/test_resource_checker_responsibility.js` so future responsibility drift is detected.

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo test -p nepl-core raw_pointer_type -- --nocapture`: 2 passed
- `cargo test -p nepl-core resource_ir_owner_check -- --nocapture`: 99 passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --all --check`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
