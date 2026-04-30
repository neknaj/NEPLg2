---
id: ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53
title: "Checked MemPtr load variant requirements lack impossible-branch refinement"
area: core
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/initialized_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53: Checked MemPtr load variant requirements lack impossible-branch refinement

## 概要

load_i32(MemPtr<i32>) correctly requires the pointee cell to be initialized when the call result is Option::Some, but the checker applies the Some requirement to every syntactic Some arm even when the wrapper guard is known to return Option::None for a statically invalid pointer such as mem_ptr_wrap 0.

## 対象

- `nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/initialized_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- Resource IR の checked MemPtr load は `Option::Some` になった場合だけ raw memory cell の初期化を要求する必要がある。
- 旧実装は wrapper guard の条件 fact を returned variant に結び付けていなかったため、`mem_ptr_wrap 0` のように `Option::Some` が静的に到達不能な呼び出しでも `Some` arm の requirement を適用していた。
- requirement 自体を弱めると未知の MemPtr からの `Option::Some` で未初期化 cell を読めてしまうため、variant 到達可能性の refinement が必要だった。

## 問題

load_i32(MemPtr<i32>) correctly requires the pointee cell to be initialized when the call result is Option::Some, but the checker applies the Some requirement to every syntactic Some arm even when the wrapper guard is known to return Option::None for a statically invalid pointer such as mem_ptr_wrap 0.

## 影響

Safe invalid-pointer handling examples that expect Option::None are rejected with resource.cell.uninit. Disabling the Some requirement would be unsound, so the missing piece is path refinement for wrapper guard facts rather than weakening the raw load precondition.

## 修正方針

Resource IR の i32 literal を typed value fact として保持し、関数 summary が returned variant と分岐条件を対応付ける。呼び出し側では実引数の raw address alias と known i32 fact から到達不能 variant を記録し、initialized-cell checker は到達不能な match arm を検査対象から外す。未知値や path merge 後の値では fact を捨て、従来どおり保守的に `Option::Some` requirement を適用する。

## 検証

- Resource IR regression として、`mem_ptr_wrap 0 -> load_i32 -> Option::None` が main の `CellUnavailable` を出さないことを確認する。
- Resource IR regression として、unknown/allocated MemPtr からの `Option::Some` path では引き続き `RawMemoryLoadCell` / `CellState::Uninit` が報告されることを確認する。
- `cargo check -p nepl-core --tests`、`cargo test -p nepl-core --test resource_ir -- --nocapture`、`trunk build`、`nodesrc/tests.js` の memory safety subset で確認する。
