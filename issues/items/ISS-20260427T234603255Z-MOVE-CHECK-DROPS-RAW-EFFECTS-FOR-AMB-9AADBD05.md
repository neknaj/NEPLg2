---
id: ISS-20260427T234603255Z-MOVE-CHECK-DROPS-RAW-EFFECTS-FOR-AMB-9AADBD05
title: "move_check drops raw effects for ambiguous function values"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T234603255Z-MOVE-CHECK-DROPS-RAW-EFFECTS-FOR-AMB-9AADBD05: move_check drops raw effects for ambiguous function values

## 概要

When a function-typed value can be one of multiple raw-effect functions, branch merge erases the function alias to None. CallIndirect then skips raw memory effect checks entirely.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `let f = if ... @clobber_a else @clobber_b` のように異なる function value が分岐合流すると、従来は alias が単一 `Option<String>` のため `None` に潰れていた。
- `CallIndirect` は callee alias が `None` の場合に raw memory effect を適用しないため、`store<LocalToken>` 済みの non-Copy raw place への callback write が D3100 にならなかった。

## 問題

When a function-typed value can be one of multiple raw-effect functions, branch merge erases the function alias to None. CallIndirect then skips raw memory effect checks entirely.

## 影響

A live non-Copy raw place can be overwritten by an indirect function call selected through a branch, bypassing D3100 raw ownership checks.

## 修正方針

Track ambiguous function-value aliases as conservative sets and apply all possible raw memory effects at indirect calls, or otherwise reject unknown raw-effect function calls conservatively.

## 対応

- function value alias と enum payload 内 function alias を候補集合として保持するように変更した。
- 分岐合流では候補集合を破棄せず union し、間接呼び出しでは候補 callee 全ての raw memory effect を適用するようにした。
- function summary の `$fnparam:*` 展開でも候補集合を伝播し、higher-order helper 経由の effect summary が単一 alias 前提に戻らないようにした。
- `tests/compiler/move_effect.n.md` に、分岐で選ばれた function value raw write が D3100 になる回帰テストを追加した。

## 検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 51/51 passed
- `cargo test -p nepl-core --test check_pipeline move_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/ambiguous-function-raw-effects.json -j 1`: 96/96 passed
