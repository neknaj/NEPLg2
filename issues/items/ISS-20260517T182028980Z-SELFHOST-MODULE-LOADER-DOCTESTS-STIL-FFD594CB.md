---
id: ISS-20260517T182028980Z-SELFHOST-MODULE-LOADER-DOCTESTS-STIL-FFD594CB
title: "selfhost module loader doctests still use ret metadata for stdout reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-18
target: tests/stdlib/neplg2_module_loader.n.md; nodesrc/test_selfhost_module_loader_report_contract.js; nodesrc/run_source_policy_regressions.js
---

# ISS-20260517T182028980Z-SELFHOST-MODULE-LOADER-DOCTESTS-STIL-FFD594CB: selfhost module loader doctests still use ret metadata for stdout reports

## 概要

tests/stdlib/neplg2_module_loader.n.md already prints deterministic std/test reports, but its doctest metadata still uses ret: 0 instead of exit_code: 0 plus stdout expectations.

## 対象

- `tests/stdlib/neplg2_module_loader.n.md`
- `nodesrc/test_selfhost_module_loader_report_contract.js`
- `nodesrc/run_source_policy_regressions.js`

## 根拠

- selfhost module loader doctest は `checks_print_report` で `Checked [...]` 形式の stdout report を出していたが、manifest は `ret: 0` だけを検証していた。
- これでは std/test report の行数、順序、成功表示、`checks_print_report` と `checks_exit_code` の責務分離が退行しても doctest が検出できない。
- module loader doctest を stdout fixture に移す途中で `item.span.file_id` の Resource IR 初期化証明不足が表面化したため、先に `ISS-20260517T182739351Z-RESOURCE-IR-REJECTS-INITIALIZED-NEST-1388C7B5` で compiler 側を修正した。

## 問題

tests/stdlib/neplg2_module_loader.n.md already prints deterministic std/test reports, but its doctest metadata still uses ret: 0 instead of exit_code: 0 plus stdout expectations.

## 影響

The selfhost module loader fixtures can regress to return-value-only validation, hiding assertion report drift between Rust and selfhost runners.

## 修正方針

Move both module loader doctests to neplg2:test[stdio, normalize_newlines] with fixed stdout reports and exit_code metadata, then add a source policy regression for the contract.

## 検証

Run the focused module loader doctests, the new source policy, issue checks, and whitespace checks.

## 対応内容

- `tests/stdlib/neplg2_module_loader.n.md` の 2 件の doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + `stdout:` へ移行した。
- 期待 stdout は `std/test` の report 本体を固定し、1 件目は 5 check、2 件目は 2 check の成功行を比較する。
- `nodesrc/test_selfhost_module_loader_report_contract.js` を追加し、doctest count、`ret:` 不使用、stdout metadata、report 件数、`checks_print_report` が `checks_exit_code` より前に呼ばれることを source policy として検査する。
- `nodesrc/run_source_policy_regressions.js` に上記 policy を追加した。

## 検証結果

- `trunk build`: passed
- `node nodesrc/test_selfhost_module_loader_report_contract.js`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_module_loader.n.md -n 1 --dist web\dist`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\neplg2_module_loader.n.md -n 2 --dist web\dist`: passed
- `node nodesrc/tests.js -i tests\stdlib\neplg2_module_loader.n.md --no-tree -o tmp\agent1-neplg2-module-loader-report-metadata.json -j 1 --dist web\dist --assert-io`: total=2, passed=2
