---
id: ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB
title: "Generic trait probe regression fixture uses raw memory in a pure helper"
area: core
status: verified
resolved: true
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/tests/neplg2.rs; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB: Generic trait probe regression fixture uses raw memory in a pure helper

## 概要

nepl-core/tests/neplg2.rs の generic raw-memory regression fixtures declared helper functions as pure while those helpers call alloc_raw, store, load, and dealloc_raw. The effect checker correctly reports PureCallsImpure, so the fixtures must express the raw-memory effect instead of weakening static checks.

## 対象

- `nepl-core/tests/neplg2.rs`
- `doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `cargo test -p nepl-core --test neplg2 generic_store_after_generic_trait_probe_preserves_struct -- --nocapture` は current branch で `Effect(PureCallsImpure)` を報告する。
- `cargo test -p nepl-core --test neplg2 generic_ -- --nocapture` では同根の fixture として `generic_intrinsic_store_load_struct_preserves_fields`、`generic_hashkey_eq_after_load_uses_concrete_impl`、`generic_hashkey_value_survives_hash_before_store`、`generic_store_after_generic_trait_probe_preserves_struct`、`generic_store_uses_nested_address_call_without_stealing_value_arg` が失敗する。
- 別 worktree の `origin/main` (`8c861744`) でも同じ test / 同じ diagnostics で再現したため、`ImplInfo` enum 化による新規破壊ではない。
- 診断対象は generic helper 内の `store` / `load` であり、trait impl identity の照合ではなく fixture の effect contract に起因する。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 5](../../doc/neplg2/abstraction_static_verification_plan.md#stage-5-monomorphize-lookup-key-typed-%E5%8C%96)

## 問題

The generic regression fixtures encoded unsafe raw-memory operations behind pure helper signatures. That makes the suite red for a correct PureCallsImpure diagnostic and risks hiding future abstraction regressions behind pressure to weaken effect checking.

## 影響

The trait/generic focused regression suite is red for an effect-boundary reason unrelated to trait application identity. Leaving it unresolved can mask future abstraction regressions or encourage weakening PureCallsImpure instead of fixing the fixture/effect contract.

## 修正方針

Do not weaken the effect checker. Mark the fixture helpers that directly call raw-memory operations as impure, keeping `probe` and other truly pure trait/generic helpers pure.

## 対応記録

- `roundtrip`、`same_after_store`、`hash_then_store`、`write_after_probe`、`write_nested` を impure function type に変更した。
- `probe` と `slot_ptr` は raw memory を直接呼ばないため pure のまま維持した。
- Stage 5 の trait/generic regression verification が effect contract の誤りで阻害されないようにした。

## 追加確認

- 2026-05-13: Stage 5 monomorphize lookup key model 変更後に `cargo test -p nepl-core --test neplg2 trait -- --nocapture` を実行し、18 件中この fixture だけが既知の `Effect(PureCallsImpure)` で失敗することを再確認した。
- 同じ run で `generic_trait_impl_method_resolves_by_trait_args`、`trait_bound_satisfied_in_generic`、`trait_method_call_with_impl_compiles` など trait lookup 関連 17 件は通過している。
- この結果は `ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14` の lookup key 変更とは別問題として扱う。

## 検証

cargo test -p nepl-core --test neplg2 generic_ -- --nocapture; cargo check -p nepl-core --tests; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo test -p nepl-core --test neplg2 generic_ -- --nocapture`: 8 passed
- `cargo check -p nepl-core --tests`: pass
