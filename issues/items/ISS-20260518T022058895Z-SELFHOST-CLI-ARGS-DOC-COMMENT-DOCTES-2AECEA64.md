---
id: ISS-20260518T022058895Z-SELFHOST-CLI-ARGS-DOC-COMMENT-DOCTES-2AECEA64
title: "selfhost CLI args doc-comment doctests hide assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/neplg2/cli/args/parse.nepl, stdlib/neplg2/cli/args/options.nepl"
---

# ISS-20260518T022058895Z-SELFHOST-CLI-ARGS-DOC-COMMENT-DOCTES-2AECEA64: selfhost CLI args doc-comment doctests hide assertion reports

## 概要

selfhost CLI args doc-comment doctests still use ret-only or unfixtureed i32 status checks, so parser/options behavior is not reported through deterministic stdout assertions.

## 対象

- `stdlib/neplg2/cli/args/parse.nepl, stdlib/neplg2/cli/args/options.nepl`

## 根拠

- `stdlib/neplg2/cli/args/parse.nepl` の doc-comment doctest 2 件は `ret: 0` だけで成功可否を表しており、成功時の parser behavior を stdout fixture として固定していなかった。
- `stdlib/neplg2/cli/args/options.nepl` には `ret: 0` だけの compile-options doctest と、metadata を持たない i32 status doctest があり、runner contract 上は「どの観測値を検査したか」が残らなかった。
- selfhost CLI args は Rust runner / selfhost runner の共通仕様になるため、exit value だけでは parser/options の差異を診断できない。

## 問題

selfhost CLI args doc-comment doctests still use ret-only or unfixtureed i32 status checks, so parser/options behavior is not reported through deterministic stdout assertions.

## 影響

Rust and selfhost doctest runners can agree on an exit value while losing assertion labels and expected/actual output, leaving CLI option behavior regressions hard to diagnose.

## 修正方針

Move the doc-comment doctests to canonical std/test TestReport stdout plus exit_code metadata, and add a source policy contract that rejects ret-only or stdout-less regressions.

## 検証

Run the new source policy and focused doctests for parse/options.

## 対応

- 4 件の doc-comment doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic stdout report へ移行した。
- `std/test::TestReport` を使い、parse success、flag、emit target、path、compile option projection を assertion label / expected / actual として stdout に固定した。
- `nodesrc/test_selfhost_cli_args_doc_report_contract.js` を追加し、対象 doctest が `ret:` や stdout-less status check へ戻らないこと、`test_report_print_stdout` と `test_report_exit_code` が分離されることを検査するようにした。
- `nodesrc/run_source_policy_regressions.js` に policy を登録した。

## 検証結果

- `node nodesrc/test_selfhost_cli_args_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib\neplg2\cli\args\parse.nepl -i stdlib\neplg2\cli\args\options.nepl --no-tree -o tmp\agent1-selfhost-cli-args-doc-report.json -j 1 --dist web\dist --assert-io`: total=4, passed=4
- `node nodesrc/issues.js check --dir issues`
