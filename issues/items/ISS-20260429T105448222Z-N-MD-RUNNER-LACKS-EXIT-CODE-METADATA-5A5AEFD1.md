---
id: ISS-20260429T105448222Z-N-MD-RUNNER-LACKS-EXIT-CODE-METADATA-5A5AEFD1
title: ".n.md runner lacks exit_code metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nodesrc/parser.ts, nodesrc/run_doctest.js, nodesrc/tests.js, nodesrc/run_test.js"
---

# ISS-20260429T105448222Z-N-MD-RUNNER-LACKS-EXIT-CODE-METADATA-5A5AEFD1: .n.md runner lacks exit_code metadata

## 概要

.n.md test metadata has ret for language return values but no exit_code field for process/test pass-fail expectations, so assertion suites cannot separate program return-value checks from exit status checks.

## 対象

- `nodesrc/parser.ts, nodesrc/run_doctest.js, nodesrc/tests.js, nodesrc/run_test.js`

## 根拠

- `.n.md` stdout assertion report 計画では、`ret:` は言語レベルの戻り値、`exit_code:` は process / runner の終了可否として分離する方針にしている。
- しかし `nodesrc/parser.ts` の metadata regex は `exit_code:` を受け付けず、`Doctest` にも `exit_code` field がなかった。
- `nodesrc/run_doctest.js` と `nodesrc/tests.js` は `expected_ret` だけを扱い、exit code expectation を表す field がなかった。
- `nodesrc/run_test.js` は runtime の `return_value` を返すが、exit-code 相当値として明示した field を持っていなかった。

## 問題

.n.md test metadata has ret for language return values but no exit_code field for process/test pass-fail expectations, so assertion suites cannot separate program return-value checks from exit status checks.

## 影響

Rust/selfhost shared doctest manifests keep overloading ret for both language return values and exit code semantics, blocking stdout-report plus exit-code based test operation.

## 修正方針

Add exit_code metadata parsing and expectation application in focused and aggregate runners; keep ret as language return value. Runners must emit an explicit exit_code result when exit_code is expected, and expectation logic must not fall back to return_value.

## 検証

Add nodesrc regression tests for parser and runner exit_code enforcement.

## 2026-04-29 解決メモ

`.n.md` metadata に `exit_code:` を追加し、`ret:` と別の field として parser / focused runner / aggregate runner へ通すようにした。

実装内容:

- `nodesrc/parser.ts` の `Doctest` と scan metadata に `exit_code` を追加した。
- metadata regex に `exit_code:` を追加し、`ret:` と同じ数値 parse 規則で `exit_code` を保持するようにした。
- `nodesrc/run_test.js` の run result に `exit_code` を追加した。短期設計として、現在の WASM/WASI runner では raw `main` return value を exit-code 相当値として出す。
- `nodesrc/run_doctest.js` に `expected_exit_code` を追加し、actual の `exit_code` が明示されている場合だけ比較するようにした。`return_value` への fallback は入れない。
- `nodesrc/tests.js` aggregate runner に `expected_exit_code` を追加し、focused runner と同じ規則で比較するようにした。
- LLVM CLI runner は process exit code を `exit_code` として出すようにした。
- `nodesrc/test_doctest_exit_code_metadata.js` を追加し、parser、focused runner、aggregate runner の `exit_code:` enforcement と `return_value` fallback 禁止を固定した。
- CI Source policy regressions に `node nodesrc/test_doctest_exit_code_metadata.js` を追加した。

この修正で runner schema と expectation は分離されたが、既存 `.n.md` fixture の `ret:` から `exit_code:` への移行と stdout report 必須化は `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` で継続する。

### 2026-04-29 検証

- `npx tsc -p nodesrc/tsconfig.json`: passed
- `node nodesrc/test_doctest_exit_code_metadata.js`: passed
- `node nodesrc/test_doctest_diag_code_metadata.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- `node nodesrc/cli.js -i doc/neplg2/nmd_assert_output_plan.md -o html=tmp/nmd-assert-output-plan-html`: generated 1 html file
- `trunk build`: passed
- `node nodesrc/test_doctest_exit_code_metadata.js` after `trunk build`: passed
- `node nodesrc/test_doctest_diag_code_metadata.js` after `trunk build`: passed
- `node nodesrc/tests.js -i tmp/doctest-exit-code-good.n.md --no-tree -o tmp/doctest-exit-code-good-manual.json -j 1 --dist web/dist`: total=1 passed=1 failed=0
