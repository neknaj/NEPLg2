---
id: ISS-20260430T124138800Z-ENTRY-MISSING-DIAGNOSTIC-SPAN-FIXTUR-9A3F17F7
title: "entry-missing diagnostic span fixture uses LLVM target under wasm runner"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: tests/compiler/compile_fail_diag_location.n.md
source: issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md
---

# ISS-20260430T124138800Z-ENTRY-MISSING-DIAGNOSTIC-SPAN-FIXTUR-9A3F17F7: entry-missing diagnostic span fixture uses LLVM target under wasm runner

## 概要

tests/compiler/compile_fail_diag_location.n.md::doctest#4 sets #target llvm even though the focused web/wasm doctest runner cannot execute the LLVM CLI-only pipeline. The fixture expects resolve.entry_function.missing_or_ambiguous but stops earlier with backend.codegen.target_requires_cli.

## 対象

- `tests/compiler/compile_fail_diag_location.n.md`

## 根拠

- `node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 4 --dist web/dist` が `backend.codegen.target_requires_cli` で失敗した。
- 同じ fixture body を `#target wasm` にすると `resolve.entry_function.missing_or_ambiguous` と `diag_span: 2:8` が通ることを一時 fixture で確認した。
- この suite は generic compile_fail diagnostic location の検証であり、LLVM CLI pipeline の検証ではない。

## 問題

tests/compiler/compile_fail_diag_location.n.md::doctest#4 sets #target llvm even though the focused web/wasm doctest runner cannot execute the LLVM CLI-only pipeline. The fixture expects resolve.entry_function.missing_or_ambiguous but stops earlier with backend.codegen.target_requires_cli.

## 影響

The diagnostic location regression does not actually verify the entry directive span in the normal web doctest runner, and full doctest runs can report a backend target error instead of the intended resolver diagnostic.

## 修正方針

Keep this as a resolver span regression by using #target wasm for the source fixture. LLVM CLI-only coverage should stay in LLVM-specific tests, not in the generic compile_fail location suite.

## 検証

Run node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 4 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`entry_missing_uses_entry_directive_span` の source target を `#target llvm` から `#target wasm` へ変更した。検証対象は entry directive の resolver diagnostic と span であり、LLVM target 固有の smoke ではないため、web doctest runner で resolver diagnostic まで到達できる target を使う。

検証:

- `node nodesrc/run_doctest.js -i tests/compiler/compile_fail_diag_location.n.md -n 4 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md --no-tree -o tmp/compile-fail-diag-location-agent1.json -j 1 --dist web/dist`: total=4, passed=4
