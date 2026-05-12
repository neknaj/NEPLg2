---
id: ISS-20260512T205049685Z-COMPILE-FAIL-DOCTESTS-LACK-STABLE-DI-326AA82A
title: "compile_fail doctests lack stable diagnostic code coverage"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-12
updated: 2026-05-13
target: "tests/compiler/*.n.md, nodesrc/test_doctest_diag_code_metadata.js, doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260512T205049685Z-COMPILE-FAIL-DOCTESTS-LACK-STABLE-DI-326AA82A: compile_fail doctests lack stable diagnostic code coverage

## 概要

Diagnostic redesign Stage D4 requires active compile_fail doctests to pin stable enum-derived diagnostic codes, but 67 compile_fail doctests still had no diag_code or diag_codes metadata.

## 対象

- `tests/compiler/*.n.md, nodesrc/test_doctest_diag_code_metadata.js, doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- 未記入

## 問題

Diagnostic redesign Stage D4 requires active compile_fail doctests to pin stable enum-derived diagnostic codes, but 67 compile_fail doctests still had no diag_code or diag_codes metadata.

## 影響

Regression tests can continue passing while a diagnostic is reclassified or collapsed into a coarse bucket, weakening enum-first diagnostic maintenance and static-check regression review.

## 修正方針

Add stable diag_code metadata to the remaining compile_fail doctests and extend source policy so new compile_fail doctests must include diag_code or diag_codes.

## 検証

Run the affected compiler doctest files, diagnostic metadata policy, source policy regressions, and issues check.

## 2026-05-13 対応結果

active doctest tree を `nodesrc/parser.js` の doctest parser で走査し、`compile_fail` tag を持つ 310 件のうち 67 件が `diag_code` / `diag_codes` を持たないことを確認した。

対応では、実際の compiler 出力に含まれる stable code を採取し、未固定だった compile_fail doctest へ `diag_codes` metadata を追加した。対象は parser、typecheck、effect、backend、Resource IR cell diagnostic を含む。

あわせて `nodesrc/test_doctest_diag_code_metadata.js` に active doctest tree の coverage policy を追加した。今後は `tests`、`stdlib/tests`、`doc`、`stdlib` 配下の `.n.md` / `.nepl` doctest で `compile_fail` に `diag_code` / `diag_codes` が無い場合、source policy が失敗する。

検証:

- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `node nodesrc/test_selfhost_diag_code_enum.js`: passed
- `node nodesrc/tests.js ... affected files ... -o tmp/agent1-diag-code-coverage-after.json -j 1 --dist web/dist`: 254 total / 252 passed / 2 failed。失敗は変更前から存在した `tests/compiler/neplg2.n.md::doctest#33` と `tests/compiler/sizeof.n.md::doctest#7` のみで、今回追加した `diag_codes` mismatch は発生していない。

追加 issue:

- [ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5](./ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5.md): affected suite の既存失敗 2 件を collection API / layout fixture 問題として分離した。
