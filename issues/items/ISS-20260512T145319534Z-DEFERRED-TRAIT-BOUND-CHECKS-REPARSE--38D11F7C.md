---
id: ISS-20260512T145319534Z-DEFERRED-TRAIT-BOUND-CHECKS-REPARSE--38D11F7C
title: "Deferred trait bound checks reparse rendered trait names"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/function_check.rs; nodesrc/test_abstraction_static_verification_policy.js"
---

# ISS-20260512T145319534Z-DEFERRED-TRAIT-BOUND-CHECKS-REPARSE--38D11F7C: Deferred trait bound checks reparse rendered trait names

## 概要

Deferred trait bound checks in function_check.rs call type_param_has_trait_bound with TraitBoundRef.name, so the checker reparses a diagnostic/display string instead of comparing the typed trait base name and type arguments already stored on the bound.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/function_check.rs; nodesrc/test_abstraction_static_verification_policy.js`

## 根拠

- `nepl-core/src/typecheck/function_check.rs` の deferred trait bound check は、`TraitBoundRef` が保持する `trait_base_name` / `trait_args` を使わず、`bound.name` を `type_param_has_trait_bound` に渡していた。
- `nepl-core/src/typecheck/traits.rs` の旧 `type_param_has_trait_bound` は `parse_trait_ref_name` により表示名から trait argument を復元していた。
- `parse_trait_ref_name` は primitive type だけを復元するため、非 primitive / nested apply / type parameter を含む trait application の静的検査根拠にできない。

## 問題

Deferred trait bound checks in function_check.rs call type_param_has_trait_bound with TraitBoundRef.name, so the checker reparses a diagnostic/display string instead of comparing the typed trait base name and type arguments already stored on the bound.

## 影響

Trait bound satisfaction can diverge for non-primitive or nested trait applications because parse_trait_ref_name only reconstructs a small primitive set. This weakens static verification for generics and trait bounds.

## 修正方針

Change type parameter bound lookup to accept trait_base_name and trait_args directly, use trait_application_matches for comparison, and keep rendered names only for diagnostics.

## 対応記録

- `type_param_has_trait_bound` を `type_param_has_trait_application_bound` に置き換え、引数を表示名ではなく `trait_base_name` と `trait_args` にした。
- `function_check.rs` の deferred trait bound satisfaction は `bound.name` を参照せず、`TraitBoundRef` の typed trait application 部分だけで type parameter bounds を照合するようにした。
- `nodesrc/test_abstraction_static_verification_policy.js` の `parse_trait_ref_name` baseline を 4 から 3 へ下げ、`function_check.rs` が `&bound.name` や旧 rendered-name lookup を使わないことを source policy で固定した。

## 検証

- `cargo test -p nepl-core generics -- --nocapture`
- `node nodesrc/tests.js -i tests/compiler/generic_impl_trait_args.n.md -i tests/compiler/generics.n.md --no-tree -o tmp/typed-trait-bound-check-generics.json -j 1 --dist web/dist`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `cargo fmt --check -p nepl-core`
