---
id: ISS-20260427T215657067Z-MOVE-CHECK-LOSES-RAW-MEMORY-EFFECTS--BDFF8DD5
title: "move_check loses raw memory effects through indirect function calls"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T215657067Z-MOVE-CHECK-LOSES-RAW-MEMORY-EFFECTS--BDFF8DD5: move_check loses raw memory effects through indirect function calls

## 概要

Function raw memory effect summaries are not propagated through CallIndirect. A higher-order helper can receive a function value such as @clobber_i32 and call it on a MemPtr, hiding raw byte writes from the caller's raw ownership state.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `CallIndirect` の move_check path は callee/args の通常 visit だけを行い、function value が指す concrete callee の `FunctionRawAliasSummary.raw_memory_effects` を caller context で適用していなかった。
- 関数サマリ生成側でも `CallIndirect` は子式の raw memory effect だけを集め、function-typed parameter 経由の callback effect を placeholder として保持していなかった。
- 修正前の inline compile_fail probe では、`apply_clobber pi @clobber_i32` が caller の live `LocalToken` raw place を `store_i32` で上書きしても `expected compile_fail, but compiled successfully` になった。

## 問題

Function raw memory effect summaries are not propagated through CallIndirect. A higher-order helper can receive a function value such as @clobber_i32 and call it on a MemPtr, hiding raw byte writes from the caller's raw ownership state.

## 影響

Higher-order stdlib/self-host helpers can hide raw memory writes, copies, and deallocations behind function-typed parameters and bypass D3100 memory-safety checks.

## 修正方針

Track known function-value aliases in move_check summaries and instantiate raw memory effects for indirect calls when the callee resolves to a known function value; keep unknown function parameters conservative as a separate effect-model problem if needed.

## 解決内容

- `MoveCheckContext` と `FunctionRawAliasSummary` に function value alias tracking を追加し、`@fn`、function-typed parameter、`let` / `set` 経由の既知 callee を保持するようにした。
- `CallIndirect` は既知 callee の raw memory effect summary を indirect call 引数へ instantiate し、direct call と同じ D3100 raw ownership 検査を実行するようにした。
- function-typed parameter がさらに別の higher-order helper へ渡される場合は `$fnparam:N` placeholder を落とさず、outer call で concrete `@fn` が渡された時点で展開するようにした。
- direct `apply_clobber pi @clobber_i32` と、多段 `forward_clobber pi @clobber_i32` の compile_fail regression を追加した。

## 検証

- `node nodesrc/run_test.js` inline probe: 修正前 compile_fail 期待が compile success
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test move_check`: 51/51 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/indirect-raw-memory-effect-summary-after.json -j 1`: 91/91 passed
- `cargo check -p nepl-core --tests`: pass
