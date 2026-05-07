---
id: ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895
title: "Examples doctests are not run by CI"
area: examples
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-07
updated: 2026-05-08
target: ".github/workflows/ci.yml, examples/*.nepl, nodesrc/tests.js"
---

# ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895: Examples doctests are not run by CI

## 概要

The CI workflow runs tests/, tutorials/, stdlib/, nm compile, and a counter emit smoke, but it does not run `node nodesrc/tests.js -i examples`. Executable examples such as rpn.nepl and rpn_legacy.nepl contain doctests for ANSI/color output and REPL behavior that can drift without a main-branch CI failure.

## 対象

- `.github/workflows/ci.yml, examples/*.nepl, nodesrc/tests.js`

## 根拠

- `.github/workflows/ci.yml` は `wasi-test` / `nmd-doctest` / `tutorials-test` / `stdlib-test` の doctest job と JSON artifact を持つが、`examples` を入力にした `nodesrc/tests.js` job がなかった。
- `examples/nm.nepl` / `examples/rpn.nepl` などは user-facing sample で、stdlib I/O、ANSI 出力、CLI args、string などの統合面を通るため、compile-only の `nm-compile` では doctest regression を検出できない。
- `node nodesrc/tests.js -i examples -o tmp/examples-ci-final.json -j 4 --dist web/dist` で examples doctest は total=32, passed=32 まで確認済みであり、CI gate に追加できる状態になった。

## 問題

The CI workflow runs tests/, tutorials/, stdlib/, nm compile, and a counter emit smoke, but it does not run `node nodesrc/tests.js -i examples`. Executable examples such as rpn.nepl and rpn_legacy.nepl contain doctests for ANSI/color output and REPL behavior that can drift without a main-branch CI failure.

## 影響

Example regressions can reach main even when user-facing sample programs are broken. This is especially risky after stdlib I/O, ANSI style, string, collection, and ownership changes because examples are the public integration surface for those APIs.

## 修正方針

Add an examples doctest CI step or matrix entry that runs `node nodesrc/tests.js -i examples -o examples-tests.json -j 4`, uploads the JSON artifact, and includes its result in the final Pages status summary.

## 検証

GitHub Actions on main executes the examples doctest job. A deliberate mismatch in an examples/*.nepl doctest fails CI and appears in the uploaded examples test JSON.

## 2026-05-08 Agent 2 修正

根本原因:

- CI の doctest coverage は `tests` / `tutorials` / `stdlib` を job と artifact に分けていたが、`examples` は `examples/nm.nepl` の compile smoke だけだった。
- `nodesrc/tests.js` は `examples` root を doctest scan 対象として扱える実装になっているため、CI workflow 側の job 欠落が問題だった。
- Pages final bundle の test artifact merge と `dist/tests/status.json` にも examples の結果が入らないため、main 上で examples doctest が壊れても status summary から発見できなかった。

修正内容:

- `.github/workflows/ci.yml` に `examples-test` job を追加し、bootstrap build artifact を使って `node nodesrc/tests.js -i examples -o examples-tests.json -j 4` を実行するようにした。
- `examples-tests.json` を `examples-tests` artifact として upload し、`pages-final-bundle` の `needs` に `examples-test` を追加した。
- Pages final bundle で examples artifact を download し、`dist/tests/examples-tests.json` へ publish するようにした。
- final `dist/tests/status.json` に `"examples_test": "${{ needs['examples-test'].result }}"` を追加した。
- `nodesrc/test_ci_examples_doctest_job.js` を追加し、CI workflow が examples doctest job、artifact、Pages merge、status summary を維持していることを source-policy regression で固定した。

検証:

- `node nodesrc/test_ci_examples_doctest_job.js`: passed
- `node nodesrc/tests.js -i examples -o tmp/examples-ci-final-after-doc.json -j 4 --dist web/dist`: total=32, passed=32
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
