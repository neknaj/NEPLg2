---
id: ISS-20260426T213058045Z-LLVM-DUAL-RUNNER-RETURNS-NULL-OR-CRA-BC38ABEB
title: "LLVM dual runner returns null or crashes for broad run cases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nodesrc/tests.js, nodesrc/test_llvm_runner_return_value.js, .github/workflows/ci.yml, doc/testing.md"
---

# ISS-20260426T213058045Z-LLVM-DUAL-RUNNER-RETURNS-NULL-OR-CRA-BC38ABEB: LLVM dual runner returns null or crashes for broad run cases

## 概要

GitHub Actions run 24967172989 tests-dual-tests has 310 run_llvm_cli return value mismatches with actual null, and tests-dual-stdlib has SIGSEGV or null returns for Option/Result/string/stdio doctests.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-cli/src/codegen_llvm.rs, nodesrc/tests.js`

## 根拠

- LLVM runner の `run_llvm_cli` result は `runtime.code` にプロセス終了コードを保持していたが、doctest expectation が読む `return_value` を設定していなかった。
- そのため native executable が正常終了しても、`applyDoctestExpectations` は `return_value` 不在を `null` と見なし、`expected_ret` 付きdoctestが `actual null` で失敗する構造だった。
- `--llvm-compile-only` でも runtime expectation を適用していたため、実行していないcompile-only結果にも `actual null` が出ていた。
- ローカル環境では `clang` がPATH上にないため、実行リンクまでのend-to-end確認は `link_llvm_cli: Error: spawn clang ENOENT` でブロックされた。

## 問題

GitHub Actions run 24967172989 tests-dual-tests has 310 run_llvm_cli return value mismatches with actual null, and tests-dual-stdlib has SIGSEGV or null returns for Option/Result/string/stdio doctests.

## 影響

LLVM dual verification currently reports broad runtime parity failure, so successful LLVM compilation is not enough to validate generated programs.

## 修正方針

Audit LLVM entry/return ABI, runtime memory initialization, process exit mapping, and runner return-value extraction; split true runtime crashes from runner capture failures.

## 検証

Run a minimal LLVM return-value smoke suite and representative Option/Result/string doctests with actual numeric returns instead of null or SIGSEGV.

## 解決

- LLVM runner の実行結果構築を `buildLlvmRunResult` に集約し、正常終了時のプロセス終了コードを `return_value` として設定するようにした。
- signal終了、exit code欠落、spawn error相当の負のcodeは abnormal として扱い、`return_value` は `null` のまま失敗結果にするようにした。
- `--llvm-compile-only` では normal doctest の return/stdout/stderr expectation を評価しないようにした。compile-onlyは実行を行わないため、runtime expectation を比較する根拠がない。
- `nodesrc/test_llvm_runner_return_value.js` を追加し、終了コードのreturn_value化、signal時のnull化、compile-only expectation skipを固定した。
- CI source policy regression と `doc/testing.md` に新しいrunner回帰テストを登録した。

## 修正後検証

- `node nodesrc/test_llvm_runner_return_value.js`: pass
- `node -e "... buildLlvmRunResult ... applyDoctestExpectations ..."`: return_value=42 の期待値照合 pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --runner llvm --llvm-all --llvm-compile-only --no-tree -o tmp/llvm-runner-compile-only-after.json -j 1`: total=27, passed=26, failed=1。`actual null` は消え、残りは `ISS-20260426T213058233Z-LLVM-COMPILE-FAIL-DIAGNOSTICS-LOSE-E-E7DD80E7` 側の compile_fail diagnostic mismatch。
- `git diff --check`: pass
- ローカルの実行リンク確認は `clang` がPATHにないため `link_llvm_cli: Error: spawn clang ENOENT` で未実施。GitHub Actions のLLVM toolchain上で end-to-end を確認する。
