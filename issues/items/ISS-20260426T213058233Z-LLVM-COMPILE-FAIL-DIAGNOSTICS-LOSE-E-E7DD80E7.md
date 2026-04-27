---
id: ISS-20260426T213058233Z-LLVM-COMPILE-FAIL-DIAGNOSTICS-LOSE-E-E7DD80E7
title: "LLVM compile_fail diagnostics lose expected ids and spans"
area: core
status: fixed
resolved: true
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-27
target: "nodesrc/tests.js, nodesrc/test_llvm_runner_return_value.js, nepl-core/src/lexer.rs, tests/compiler/compile_fail_diag_location.n.md, tests/compiler/codegen_diagnostics.n.md, tests/compiler/move_effect.n.md"
---

# ISS-20260426T213058233Z-LLVM-COMPILE-FAIL-DIAGNOSTICS-LOSE-E-E7DD80E7: LLVM compile_fail diagnostics lose expected ids and spans

## 概要

GitHub Actions run 24967172989 llvm-dual-tests reports compile_fail diagnostic id/span mismatches for codegen_diagnostics and compile_fail_diag_location under compile_llvm_cli.

## 対象

- `nodesrc/tests.js, nepl-core/src/compiler.rs, nepl-core/src/codegen_llvm.rs, tests/compiler/compile_fail_diag_location.n.md`

## 根拠

- LLVM CLI compile_fail は `Error: ... [D3001] ... (file=0, start=38, end=50)` のような compact span 形式を返すが、`nodesrc/tests.js` は `--> file:line:col` 形式だけを抽出していたため、expected `diag_span` がすべて missing になっていた。
- `#entry main` の lexer token span が entry 名ではなく directive 全体を指していたため、`TypeEntryFunctionMissingOrAmbiguous` の primary span が `main` ではなく `#entry` 先頭になっていた。
- `codegen_diagnostics.n.md` の invalid raw wasm test は WASM backend raw line parser の `D4004` を確認するケースで、LLVM targetでは raw body target mismatch `D3095` が正しいため、LLVM runnerで同じID期待を適用するべきではなかった。
- `move_effect.n.md` の Writer/std streamio move test は std target と streamio import 前提の診断で、LLVM targetのstdlib loweringでは先に別診断へ落ちるため、LLVM compile_fail diagnostic比較の対象にしない方が正しい。

## 問題

GitHub Actions run 24967172989 llvm-dual-tests reports compile_fail diagnostic id/span mismatches for codegen_diagnostics and compile_fail_diag_location under compile_llvm_cli.

## 影響

Strict dual testing cannot distinguish real diagnostic regressions from LLVM target diagnostic rewriting, and compile_fail coverage becomes noisy.

## 修正方針

Preserve diagnostic IDs/spans through LLVM compile-fail mode or make the dual runner compare the original typecheck diagnostics before LLVM-specific wrapping.

## 検証

Run tests/compiler/codegen_diagnostics.n.md and tests/compiler/compile_fail_diag_location.n.md with --runner llvm --llvm-all and confirm expected ids/spans match.

## 解決

- `nodesrc/tests.js` の compile_fail span抽出で、LLVM CLI の compact span `(file=0, start=..., end=...)` を doctest source から `/virtual/entry.nepl:line:col` へ復元するようにした。
- `#entry` directive の lexer spanを directive全体ではなく entry名部分にした。
- WASM raw body parser専用の `D4004` test と std streamio move専用の compile_fail testに `skip_llvm` を付け、target固有診断をLLVM runnerで誤比較しないようにした。
- `nodesrc/test_llvm_runner_return_value.js` に compact span復元の回帰テストを追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `trunk build`: pass
- `node nodesrc/test_llvm_runner_return_value.js`: pass
- `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md -i tests/compiler/codegen_diagnostics.n.md -i tests/compiler/move_effect.n.md --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-compile-fail-diag-final.json -j 1`: total=32, passed=32
- `git diff --check`: pass（CRLF 変換警告のみ）
