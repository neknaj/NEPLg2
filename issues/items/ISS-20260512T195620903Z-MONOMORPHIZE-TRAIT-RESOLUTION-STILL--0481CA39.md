---
id: ISS-20260512T195620903Z-MONOMORPHIZE-TRAIT-RESOLUTION-STILL--0481CA39
title: "Monomorphize trait resolution still accepts split trait names"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T195620903Z-MONOMORPHIZE-TRAIT-RESOLUTION-STILL--0481CA39: Monomorphize trait resolution still accepts split trait names

## 概要

Monomorphize call rewriting receives typed HirTraitApplication and HirTraitMethodId, but resolve_trait_impl_name immediately splits them back into trait_name, trait_args, and method string parameters.

## 対象

- `nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- HIR `FuncRef::Trait` は `HirTraitApplication` と `HirTraitMethodId` を保持するようになっている。
- monomorphize lookup key も `MonoTraitApplication` / `MonoTraitMethodId` / `MonoTraitLookupKey` へ移行済みである。
- しかし resolver 入口だけが `resolve_trait_impl_name(trait_name, trait_args, method, ...)` として split string/args を再受け取りしていた。

## 問題

Monomorphize call rewriting receives typed HirTraitApplication and HirTraitMethodId, but resolve_trait_impl_name immediately splits them back into trait_name, trait_args, and method string parameters.

## 影響

The lookup keys are typed internally, but call-site resolution can still grow new string authority at the HIR-to-monomorphize boundary. This weakens Stage 5 typed trait identity enforcement.

## 修正方針

Rename the resolver to resolve_trait_impl, accept HirTraitApplication and HirTraitMethodId directly, convert to MonoTraitApplication/MonoTraitMethodId inside the resolver, and extend abstraction policy to reject the old split-name resolver.

## 対応記録

- `resolve_trait_impl_name` を `resolve_trait_impl` に改名し、`&HirTraitApplication` と `&HirTraitMethodId` を直接受け取るようにした。
- resolver 内部で `MonoTraitApplication::from_hir` と `MonoTraitMethodId::from_name(method.as_str())` に変換し、call site で trait name / method string を分解しない形にした。
- `MonoTraitMethodKey` は resolver 内の `MonoTraitApplication.trait_id` から構築する。
- abstraction source policy に旧 `resolve_trait_impl_name` 再導入禁止と typed resolver signature を追加した。

## 検証

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
