---
id: ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB
title: "Generic trait probe regression fixture uses raw memory in a pure helper"
area: core
status: open
resolved: false
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/tests/neplg2.rs; nepl-core/src/typecheck/effects.rs; doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB: Generic trait probe regression fixture uses raw memory in a pure helper

## 概要

nepl-core/tests/neplg2.rs::generic_store_after_generic_trait_probe_preserves_struct fails on origin/main and the current branch because write_after_probe is declared pure while it calls alloc_raw, store, load, and dealloc_raw.

## 対象

- `nepl-core/tests/neplg2.rs; nepl-core/src/typecheck/effects.rs; doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- `cargo test -p nepl-core --test neplg2 generic_store_after_generic_trait_probe_preserves_struct -- --nocapture` は current branch で `Effect(PureCallsImpure)` を報告する。
- 別 worktree の `origin/main` (`8c861744`) でも同じ test / 同じ diagnostics で再現したため、`ImplInfo` enum 化による新規破壊ではない。
- 診断対象は `write_after_probe__T_V__T__pure_Point_i32` 内の `store` / `store` / `load` であり、trait impl identity の照合ではなく fixture の effect contract に起因する。

## 問題

nepl-core/tests/neplg2.rs::generic_store_after_generic_trait_probe_preserves_struct fails on origin/main and the current branch because write_after_probe is declared pure while it calls alloc_raw, store, load, and dealloc_raw.

## 影響

The trait/generic focused regression suite is red for an effect-boundary reason unrelated to trait application identity. Leaving it untracked can mask future abstraction regressions or encourage weakening PureCallsImpure instead of fixing the fixture/effect contract.

## 修正方針

Do not weaken the effect checker. Update the fixture or helper signature to express the raw-memory effect explicitly, or move the raw storage path behind an audited boundary if the test intends to validate pure public behavior.

## 追加確認

- 2026-05-13: Stage 5 monomorphize lookup key model 変更後に `cargo test -p nepl-core --test neplg2 trait -- --nocapture` を実行し、18 件中この fixture だけが既知の `Effect(PureCallsImpure)` で失敗することを再確認した。
- 同じ run で `generic_trait_impl_method_resolves_by_trait_args`、`trait_bound_satisfied_in_generic`、`trait_method_call_with_impl_compiles` など trait lookup 関連 17 件は通過している。
- この結果は `ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14` の lookup key 変更とは別問題として扱う。

## 検証

cargo test -p nepl-core --test neplg2 generic_store_after_generic_trait_probe_preserves_struct -- --nocapture; cargo test -p nepl-core trait -- --nocapture
