---
id: ISS-20260517T180445599Z-SELFHOST-PARSER-DOCTEST-STILL-USES-R-6B2C918C
title: "selfhost parser doctest still uses ret metadata for stdout report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: "tests/stdlib/neplg2_parser.n.md, nodesrc/test_selfhost_parser_report_contract.js"
---

# ISS-20260517T180445599Z-SELFHOST-PARSER-DOCTEST-STILL-USES-R-6B2C918C: selfhost parser doctest still uses ret metadata for stdout report

## 概要

tests/stdlib/neplg2_parser.n.md already prints a deterministic std/test report, but its .n.md metadata still uses ret: 0 instead of exit_code: 0 plus stdout expectation. This keeps the runner contract ambiguous for Rust/selfhost shared tests.

## 対象

- `tests/stdlib/neplg2_parser.n.md, nodesrc/test_selfhost_parser_report_contract.js`

## 根拠

- `tests/stdlib/neplg2_parser.n.md::doctest#1` は `checks_print_report` と `checks_exit_code` を呼び、実行時には deterministic な `std/test` report を stdout へ出している。
- しかし metadata は `ret: 0` のままだったため、fixture が「stdout report を仕様として固定する test」なのか「戻り値だけを確認する test」なのかを parser / runner contract 上で区別できなかった。
- 同系統の selfhost lexer / type arena fixture は既に `neplg2:test[stdio, normalize_newlines]` + `stdout:` + `exit_code:` へ移行済みであり、parser だけが同じ report contract から外れていた。

## 問題

tests/stdlib/neplg2_parser.n.md already prints a deterministic std/test report, but its .n.md metadata still uses ret: 0 instead of exit_code: 0 plus stdout expectation. This keeps the runner contract ambiguous for Rust/selfhost shared tests.

## 影響

The parser selfhost fixture can silently regress to return-value-only validation, losing assertion labels and stdout report compatibility checks.

## 修正方針

Move the parser fixture to neplg2:test[stdio, normalize_newlines] with stdout report and exit_code metadata, and add a source policy regression that rejects ret metadata for this fixture.

## 検証

Run the focused doctest, the new source policy, source policy registry, issue checks, and whitespace checks.

## 対応結果

- `tests/stdlib/neplg2_parser.n.md::doctest#1` を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + `stdout: mlstr:` へ移行した。
- stdout には 21 件の `std/test` assertion report を固定し、`ret:` を削除した。
- `nodesrc/test_selfhost_parser_report_contract.js` を追加し、この fixture が `ret:` へ戻らないこと、stdout report と tag が維持されること、`checks_print_report` から `checks_exit_code` へつながることを source policy にした。
- `nodesrc/run_source_policy_regressions.js` に parser report contract を登録した。

検証:

- `node nodesrc/test_selfhost_parser_report_contract.js`: pass
- `node nodesrc/run_doctest.js -i tests\\stdlib\\neplg2_parser.n.md -n 1 --dist web\\dist`: pass
- `node nodesrc/tests.js -i tests\\stdlib\\neplg2_parser.n.md --no-tree -o tmp\\agent1-neplg2-parser-report-metadata.json -j 1 --dist web\\dist --assert-io`: total=1, passed=1
- `node nodesrc/issues.js check --dir issues`: pass

補足:

- `node nodesrc/run_source_policy_regressions.js` は別件の stale policy `nodesrc/test_resource_checker_responsibility.js` で失敗した。この問題は `ISS-20260517T180734291Z-RESOURCE-CHECKER-SOURCE-POLICY-STILL-8BAE7A40` として分離した。
