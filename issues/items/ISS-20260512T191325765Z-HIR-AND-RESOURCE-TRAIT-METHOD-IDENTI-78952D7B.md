---
id: ISS-20260512T191325765Z-HIR-AND-RESOURCE-TRAIT-METHOD-IDENTI-78952D7B
title: "HIR and Resource trait method identity still uses raw String"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/hir.rs; nepl-core/src/resource/model.rs; nepl-core/src/resource/trait_identity.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/passes/drop_insertion.rs; nepl-core/src/monomorphize.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/dump.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T191325765Z-HIR-AND-RESOURCE-TRAIT-METHOD-IDENTI-78952D7B: HIR and Resource trait method identity still uses raw String

## 概要

HIR and Resource IR trait applications now carry typed trait ids, but FuncRef::Trait and ResourceCallTarget::Trait still store method identity as method: String. This leaves trait method lookup identity untyped across HIR, Resource IR, and monomorphize boundaries.

## 対象

- `nepl-core/src/hir.rs; nepl-core/src/resource/model.rs; nepl-core/src/resource/trait_identity.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/passes/drop_insertion.rs; nepl-core/src/monomorphize.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/dump.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 は、typecheck / HIR / monomorphize / Resource IR の trait application と method identity を typed model で接続し、表示文字列は diagnostic/display 境界だけに限定する方針である。
- `FuncRef::Trait` と `ResourceCallTarget::Trait` が `method: String` を保持すると、trait identity を newtype 化しても method identity だけが raw display/name payload として残る。

## 問題

HIR and Resource IR trait applications now carry typed trait ids, but FuncRef::Trait and ResourceCallTarget::Trait still store method identity as method: String. This leaves trait method lookup identity untyped across HIR, Resource IR, and monomorphize boundaries.

## 影響

Trait identity is typed but method identity can still be mixed with rendered diagnostic names by convention. Source policy cannot catch a regression that keeps typed trait ids while reintroducing raw method authority.

## 修正方針

Introduce HirTraitMethodId and ResourceTraitMethodId newtypes, use them in FuncRef::Trait and ResourceCallTarget::Trait, expose string names only through as_str/display boundaries, and extend abstraction source policy to reject method: String in those intermediate representations.

## 対応記録

- `HirTraitMethodId` を追加し、`FuncRef::Trait.method` を raw `String` から newtype へ移行した。
- `ResourceTraitMethodId` を追加し、`ResourceCallTarget::Trait.method` を raw `String` から newtype へ移行した。Resource IR の trait identity type は `resource/trait_identity.rs` へ分離し、`resource/model.rs` の責務肥大化も避けた。
- trait method call lowering、drop insertion、monomorphize unresolved report、Resource IR lowering / dump / i32 helper facts、LLVM/WASM codegen diagnostic を `as_str()` 境界へ追従させた。
- source policy に HIR / Resource IR の method identity newtype と `method: String` 再導入禁止を追加した。

## 検証

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
