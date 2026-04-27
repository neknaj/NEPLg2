---
id: ISS-20260427T233905940Z-MOVE-CHECK-LOSES-RAW-PROVENANCE-FOR--98EEA2E1
title: "move_check loses raw provenance for signed pointer offsets"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T233905940Z-MOVE-CHECK-LOSES-RAW-PROVENANCE-FOR--98EEA2E1: move_check loses raw provenance for signed pointer offsets

## 概要

`raw_memory_place_key` treated raw `add` offsets as non-negative and did not model `sub base offset`. Expressions like `sub base size_of<T>` therefore lost base provenance and could bypass raw non-Copy ownership checks.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- Reproduction added during investigation:
  - `let q <i32> sub base size_of<LocalToken>`
  - `store<LocalToken> q ...`
  - `load<LocalToken> q`
  - `load<LocalToken> sub base size_of<LocalToken>`
- Before the fix, the temporary doctest expected D3100 but compiled successfully.
- `core/math` `sub` lowers through raw backend bodies, so function raw alias summaries cannot recover the address relationship unless `move_check` recognizes signed pointer arithmetic directly.

## 問題

`raw_memory_place_key` only combined base provenance with `non_negative_i32_const_from_value` for `add`. A negative constant offset, or the equivalent `sub base offset`, became untracked or a different raw place. This allowed the same raw cell to be referenced through two syntactically different address expressions.

## 影響

The same raw memory cell can be accessed through a negative-offset expression and a let-bound alias without D3100, allowing duplicate non-Copy ownership from raw memory.

## 修正方針

- Represent raw offsets as signed values.
- Normalize `add base offset` to `base + offset`, including negative constant offsets.
- Normalize `sub base offset` to `base - offset`.
- Keep unknown offsets as `base+?` so overlap remains conservative.

## 検証

- `cargo fmt --check`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-negative-offset-provenance.json -j 1`: 95/95 passed
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 51/51 passed
- `cargo test -p nepl-core --test check_pipeline move_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: pass
- `cargo check -p nepl-core --tests`: pass
