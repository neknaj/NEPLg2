---
id: ISS-20260517T083417129Z-SHADOW-WARNINGS-DEPEND-ON-HARDCODED--58058C13
title: "shadow warnings depend on hardcoded important stdlib symbol list"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/typecheck/binding_rules.rs; nepl-language/src/lib.rs; nepl-web/src/lib.rs; tests/compiler/tree/07_shadow_warning_policy.js; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T083417129Z-SHADOW-WARNINGS-DEPEND-ON-HARDCODED--58058C13: shadow warnings depend on hardcoded important stdlib symbol list

## 概要

typecheck and name-resolution analysis classify shadow warnings through a hardcoded important stdlib symbol list such as print/add/map/len. This is not source-derived proof and it diverges across nepl-core, nepl-language, and nepl-web.

## 対象

- `nepl-core/src/typecheck/binding_rules.rs; nepl-language/src/lib.rs; nepl-web/src/lib.rs; tests/compiler/tree/07_shadow_warning_policy.js; nodesrc/test_static_check_boundary_responsibility.js; doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nepl-core/src/typecheck/binding_rules.rs` had a local `is_important_shadow_symbol` allowlist for names such as `print`, `add`, and comparison helpers.
- `nepl-language/src/lib.rs` and `nepl-web/src/lib.rs` carried their own shadow warning allowlists, and the web/editor option exposed the old `warn_important_shadow` model.
- A name like `print` could warn even when no outer definition was actually shadowed, while future stdlib symbols would not warn unless every allowlist was updated.

## 問題

typecheck and name-resolution analysis classify shadow warnings through a hardcoded important stdlib symbol list such as print/add/map/len. This is not source-derived proof and it diverges across nepl-core, nepl-language, and nepl-web.

## 影響

The compiler can warn about names that are not actually shadowing a definition while missing future stdlib APIs unless the allowlist is manually updated. This violates the policy that checks should be based on source facts such as actual bindings and noshadow declarations rather than stdlib name allowlists.

## 修正方針

Remove the important stdlib symbol allowlist. Emit shadow warnings from actual outer binding evidence only, keep noshadow as the authoritative source-level protection, and rename editor options/policy away from important-shadow wording.

## 対応

- Removed the hardcoded important-symbol shadow allowlist from core typecheck, nepl-language, and nepl-web.
- Renamed the diagnostic from `resolve.shadow.important_symbol` / `ShadowImportantSymbol` to `resolve.shadow.outer_definition` / `ShadowOuterDefinition`.
- Changed shadow warning emission to require actual `lookup_outer_defined` / existing outer definition evidence.
- Changed editor/name-resolution warning severity so same-scope redefinitions stay informational and only an actual outer-scope definition produces a warning.
- Renamed the editor-facing option and policy field from `warn_important_shadow` to `warn_shadow`; old compatibility spelling is intentionally not kept.
- Updated tree tests so the warning case shadows an actual outer local binding and the no-warning case defines a fresh local symbol.
- Added source policy checks that reject reintroducing the old important-shadow allowlist or diagnostic id.

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo fmt -p nepl-language --check`
- `cargo check -p nepl-core`
- `cargo check -p nepl-language`
- `cargo check --manifest-path nepl-web/Cargo.toml`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node tests/compiler/tree/run.js` (`total=20`, `passed=20`, `failed=0`, `errored=0`)
- `node nodesrc/tests.js -i tests/compiler/shadowing.n.md --no-tree -o tmp/agent1-shadowing-tests.json -j 1` (`total=27`, `passed=27`, `failed=0`, `errored=0`)
- `node nodesrc/issues.js index --dir issues`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
