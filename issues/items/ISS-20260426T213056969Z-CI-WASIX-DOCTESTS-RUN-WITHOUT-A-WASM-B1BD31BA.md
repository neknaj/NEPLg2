---
id: ISS-20260426T213056969Z-CI-WASIX-DOCTESTS-RUN-WITHOUT-A-WASM-B1BD31BA
title: "CI wasix doctests run without a wasmer executable"
area: cli
status: verified
resolved: true
priority: P1
type: test
created: 2026-04-26
updated: 2026-04-27
target: ".github/workflows/ci.yml, nodesrc/run_test.js, tests/stdlib/features_tui.n.md"
---

# ISS-20260426T213056969Z-CI-WASIX-DOCTESTS-RUN-WITHOUT-A-WASM-B1BD31BA: CI wasix doctests run without a wasmer executable

## 概要

GitHub Actions run 24967172989 fails tests/stdlib/features_tui.n.md doctest#1-#4 in nmd-doctest, wasi-test, and llvm-dual-tests with Error: spawn wasmer ENOENT. Existing Wasmer issues cover option compatibility and tty imports, but not the missing executable in CI.

## 対象

- `.github/workflows/ci.yml, nodesrc/run_test.js, tests/stdlib/features_tui.n.md`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 fails tests/stdlib/features_tui.n.md doctest#1-#4 in nmd-doctest, wasi-test, and llvm-dual-tests with Error: spawn wasmer ENOENT. Existing Wasmer issues cover option compatibility and tty imports, but not the missing executable in CI.

## 影響

All #target wasix doctests are red before compiler/runtime checks run, and TUI regressions are masked by CI environment setup.

## 修正方針

Install a pinned Wasmer in every workflow job that can run #target wasix tests, or make the test runner detect missing wasmer and route supported WASIX cases through the Node fallback with an explicit diagnostic.

## 検証

Run GitHub Actions nmd-doctest and wasi-test and confirm features_tui doctest#1-#4 no longer fail with spawn wasmer ENOENT.

## 対応記録

- 原因: `nodesrc/run_test.js` の `#target wasix` 経路は必ず `wasmer run` を起動し、`spawn wasmer ENOENT` を通常の runtime trap として返すだけだった。CI の nmd-doctest / wasi-test / llvm-dual-test は Wasmer を bootstrap していないため、WASIX TUI doctest は compiler/runtime 本体へ到達する前に環境依存で落ちていた。
- 対応: Wasmer spawn error の `ENOENT` を明示的に検出し、既存の Node WASI + WASIX TTY host import fallback へ流すようにした。fallback result には runner 名と fallback reason を残し、fallback 側が失敗した場合も「Wasmer が無いので fallback した」ことが診断に残る。
- 回帰: `nodesrc/test_run_test_wasix_missing_wasmer_fallback.js` を追加し、`WASMER_BIN` が存在しないときでも WASIX runner が Node fallback で最小 wasm を実行できることを固定した。CI の source policy regression と `doc/testing.md` / `nodesrc/README.n.md` も更新した。
- 検証:
  - `node --check nodesrc/run_test.js`: pass
  - `node nodesrc/test_run_test_wasi_tmp_dir.js`: pass
  - `node nodesrc/test_run_test_wasix_missing_wasmer_fallback.js`: pass
  - `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-wasmer-fallback.json -j 1`: `total=4`, `passed=4`
  - `WASMER_BIN=<missing> node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-missing-wasmer.json -j 1`: `total=4`, `passed=4`
