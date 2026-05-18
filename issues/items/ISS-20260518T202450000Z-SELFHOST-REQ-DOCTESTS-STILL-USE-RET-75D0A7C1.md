---
id: ISS-20260518T202450000Z-SELFHOST-REQ-DOCTESTS-STILL-USE-RET-75D0A7C1
title: "selfhost_req doctests still use ret-only requirement checks"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_req.n.md, nodesrc/test_selfhost_req_report_contract.js, nodesrc/run_source_policy_regressions.js"
---

# ISS-20260518T202450000Z-SELFHOST-REQ-DOCTESTS-STILL-USE-RET-75D0A7C1: selfhost_req doctests still use ret-only requirement checks

## 概要

`tests/stdlib/selfhost_req.n.md` の 6 doctest が、self-host 実装の要件確認であるにもかかわらず `ret:` だけで検査結果を表していた。

## 対象

- `tests/stdlib/selfhost_req.n.md`
- `nodesrc/test_selfhost_req_report_contract.js`
- `nodesrc/run_source_policy_regressions.js`

## 根拠

- self-host 実装前の要件確認は、どの機能が期待値どおりに動いたかを stdout 上の deterministic assertion report として残す必要がある。
- `ret:` は言語レベルの戻り値確認と process exit-code 代用が混ざるため、`.n.md` runner / selfhost runner の互換性検査として不十分である。
- 既存の `.n.md` report 化方針では、`std/test` の `TestReport` を stdout に出し、`exit_code:` は可否だけを表す。

## 問題

`selfhost_req` は filesystem failure handling、byte buffer、string helper、string-key map、StringBuilder、trait extension を確認する gate だが、旧形式では `0` / `222` / `10` / `20` / `5` などの戻り値だけが期待値になっていた。これでは失敗時にどの要件が壊れたかが fixture 差分から分からず、self-host runner と Rust runner の stdout report 互換も固定できない。

## 影響

self-host 実装に移る前の要求仕様テストが、成功時の assertion detail を持たないまま残る。将来 runner を差し替えたとき、exit code が一致しても report format や assertion label の退行を検出できない。

## 修正方針

6 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行する。各 doctest は `std/test::TestReport` に要件ごとの assertion label と expected/actual を出力し、最後に `test_report_exit_code` を返す。source policy は同 file が `ret:` へ戻らないこと、stdout report を固定すること、report 出力後に exit code を返すことを監視する。

## 解決

- `tests/stdlib/selfhost_req.n.md` の 6 doctest を `ret:` 依存から `TestReport` stdout + `exit_code: 0` へ移行した。
- `nodesrc/test_selfhost_req_report_contract.js` を追加し、doctest count、tags、stdout、`ret:` 不使用、`test_report_print_stdout -> test_report_exit_code` の順序を固定した。
- `nodesrc/run_source_policy_regressions.js` に同 policy を登録した。

## 関連 issue

- [.n.md tests rely on return values instead of stdout assertion reports](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)

## 検証結果

- `node nodesrc/test_selfhost_req_report_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/agent1-selfhost-req-report-contract.json -j 1 --dist web/dist --assert-io`: total=6, passed=6
- `node nodesrc/issues.js check`: passed
