---
id: ISS-20260427T214055047Z-MOVE-CHECK-IGNORES-RAW-MEMORY-WRITES-417A7103
title: "move_check ignores raw memory writes hidden behind helper functions"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T214055047Z-MOVE-CHECK-IGNORES-RAW-MEMORY-WRITES-417A7103: move_check ignores raw memory writes hidden behind helper functions

## 概要

move_check checks raw byte writes at direct call sites, but function calls only propagate return raw aliases. A helper that takes MemPtr<i32> and calls store_i32 can overwrite a caller's live non-Copy raw place without D3100.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` の direct call path は raw memory operation を `visit_raw_memory_call` で検査するが、`FunctionRawAliasSummary` は戻り値の raw alias だけを保持していた。
- `block_raw_alias_summary` は関数本体の最終式の alias summary だけを返し、途中行の `let r = store_i32 p 0` のような raw memory 副作用を caller に伝播しなかった。
- 修正前の `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/function-raw-byte-write-summary-before.json -j 1` では、追加した helper 経由 raw byte write regression が `expected compile_fail, but compiled successfully` になった。

## 問題

move_check checks raw byte writes at direct call sites, but function calls only propagate return raw aliases. A helper that takes MemPtr<i32> and calls store_i32 can overwrite a caller's live non-Copy raw place without D3100.

## 影響

stdlib/self-host helper functions can hide raw storage mutations from the caller, corrupting initialized non-Copy payloads even after direct raw write checks were added.

## 修正方針

Extend function raw summaries to include raw memory write/copy/dealloc/realloc effects in terms of function parameters, instantiate those effects at call sites, and run the same raw ownership checks in the caller context.

## 解決内容

- `FunctionRawAliasSummary` に raw memory effect summary を追加し、load/store/dealloc/realloc/bulk copy/byte write を関数引数由来の raw place key として記録するようにした。
- user function call では callee summary を caller 引数へ instantiate し、direct raw call と同じ `check_raw_non_copy_*` 系の検査を caller context で実行するようにした。
- `block_raw_alias_summary` は戻り値 alias とブロック内 raw memory effect を分離し、途中行の副作用も関数サマリへ蓄積するようにした。
- `if` 条件、`while` 条件/本体、`match` scrutinee/arm、nested block の raw memory effect も式サマリへ含め、制御式に隠した副作用も caller へ伝播するようにした。
- raw memory effect を持つ user function call は iterative visit の対象外にし、caller context での副作用検査を省略しないようにした。
- `MemPtr<i32>` を受け取る helper 関数と、`if` 条件内 helper 関数が caller の live non-Copy raw place を `store_i32` で上書きできない compile_fail regression を追加した。

## 検証

- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/function-raw-byte-write-summary-before.json -j 1`: 修正前 88 件中 1 件失敗
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test move_check`: 51/51 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/function-raw-byte-write-summary-after.json -j 1`: 89/89 passed
- `cargo check -p nepl-core --tests`: pass
