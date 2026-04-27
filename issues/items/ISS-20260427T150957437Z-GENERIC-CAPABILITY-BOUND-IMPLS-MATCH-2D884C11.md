---
id: ISS-20260427T150957437Z-GENERIC-CAPABILITY-BOUND-IMPLS-MATCH-2D884C11
title: "generic capability-bound impls match non-capable actual types"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/types.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T150957437Z-GENERIC-CAPABILITY-BOUND-IMPLS-MATCH-2D884C11: generic capability-bound impls match non-capable actual types

## 概要

TypeCtx::type_pattern_matches treats an unbound pattern type variable as matching any actual type even when that variable carries Copy or Drop capability bounds. A generic impl such as impl<.T: Copy> Copy for Option<.T> can therefore be considered a match for Option<NonCopy>.

## 対象

- `nepl-core/src/types.rs, tests/compiler/move_effect.n.md`

## 根拠

- `TypeCtx::type_pattern_matches` の `TypeKind::Var` 分岐は、pattern 側の型変数が未束縛なら capability bound を確認せず actual 型へ割り当てていた。
- `impl<.T: Copy> Copy for Option<.T>` が `Option<LocalToken>` にも一致すると、move checker は `Option<LocalToken>` の二重利用を拒否できない。

## 問題

TypeCtx::type_pattern_matches treats an unbound pattern type variable as matching any actual type even when that variable carries Copy or Drop capability bounds. A generic impl such as impl<.T: Copy> Copy for Option<.T> can therefore be considered a match for Option<NonCopy>.

## 影響

The move checker can classify wrappers containing owning non-Copy values as Copy or Drop-capable through an unsatisfied generic impl. That permits shallow copies and can invalidate memory-safety assumptions around ownership, Drop insertion, and raw storage cleanup.

## 修正方針

Make impl target pattern matching validate capability bounds when binding generic pattern variables. Copy-bound variables must match only actual types that satisfy Copy, and Drop-bound variables must match only actual types that satisfy Drop. Preserve generic Copy impls for truly Copy actual types.

## 検証

Add compiler regression tests that reject reusing Option<LocalToken> through impl<.T: Copy> Copy for Option<.T>, while keeping Option<i32> reusable through the same impl.

## 対応結果

- `TypeCtx::type_pattern_matches` が generic pattern variable を actual 型へ対応付ける前に `copy_cap` / `drop_cap` を評価するようにした。
- `tests/compiler/move_effect.n.md` に `Option<LocalToken>` の再利用 compile_fail と、`Option<i32>` の再利用正常系を追加した。

## 実施した検証

- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/capability-bound-impl-matching.json -j 1`: `total=33`, `passed=33`
