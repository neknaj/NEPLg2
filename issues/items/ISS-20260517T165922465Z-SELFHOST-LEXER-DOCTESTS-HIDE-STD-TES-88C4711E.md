---
id: ISS-20260517T165922465Z-SELFHOST-LEXER-DOCTESTS-HIDE-STD-TES-88C4711E
title: "selfhost lexer doctests hide std/test reports behind ret metadata"
area: selfhost
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-18
target: tests/stdlib/neplg2_lexer.n.md
---

# ISS-20260517T165922465Z-SELFHOST-LEXER-DOCTESTS-HIDE-STD-TES-88C4711E: selfhost lexer doctests hide std/test reports behind ret metadata

## 概要

self-host lexer doctest は 13 件すべてで `checks_print_report` を呼んでいるにもかかわらず、manifest は `neplg2:test` + `ret: 0` のままだった。stdout fixture、`exit_code:` metadata、`stdio` / `normalize_newlines` tag がないため、assertion report は検査契約ではなく incidental な runtime output になっていた。

## 対象

- `tests/stdlib/neplg2_lexer.n.md`

## 根拠

- `tests/stdlib/neplg2_lexer.n.md` の 13 doctest はすべて `std/test` を import し、`checks_print_report` と `checks_exit_code` を呼んでいた。
- 変更前の focused run では 13 件とも stdout に `Checked [...]` report を出していたが、fixture には stdout が固定されていなかった。
- 13 件すべてが `ret: 0` に依存しており、言語戻り値と process exit-code の責務が manifest 上で分離されていなかった。

## 問題

self-host lexer doctest は 13 件すべてで `checks_print_report` を呼んでいるにもかかわらず、manifest は `neplg2:test` + `ret: 0` のままだった。stdout fixture、`exit_code:` metadata、`stdio` / `normalize_newlines` tag がないため、assertion report は検査契約ではなく incidental な runtime output になっていた。

## 影響

Rust runner と selfhost runner が exit code だけ一致しても、lexer assertion の件数、report format、出力順が silent に drift し得る。CI artifact でも stdout diff からどの lexer 観測が壊れたかを追跡しにくい。

## 修正方針

各 lexer doctest を `neplg2:test[stdio, normalize_newlines]` + stdout report + `exit_code: 0` として固定し、`ret:` を削除する。さらに source policy を追加し、この file が quiet exit-code-only metadata へ戻らないようにする。

## 検証

source policy と `neplg2_lexer` focused doctest を `--assert-io` 付きで実行する。

## 修正内容

- `tests/stdlib/neplg2_lexer.n.md` の 13 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- `ret:` を削除し、lexer の report stdout と process exit-code の責務を manifest 上で分離した。
- `nodesrc/test_selfhost_lexer_report_contract.js` を追加し、13 件の doctest count、stdout report 件数、`ret:` 不使用、`exit_code: 0`、report print と exit code derivation の順序を固定した。
- `nodesrc/run_source_policy_regressions.js` に同 policy を登録した。

## 検証結果

- `node nodesrc/test_selfhost_lexer_report_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-neplg2-lexer-report-metadata.json -j 1 --dist web/dist --assert-io`: total=13, passed=13
