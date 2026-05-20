---
id: ISS-20260520T035443109Z-SELFHOST-PARSER-REPORT-CONTRACT-EXPE-2F0D52E2
title: "selfhost parser report contract expects stale assertion count"
area: selfhost
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/test_selfhost_parser_report_contract.js, tests/stdlib/neplg2_parser.n.md"
---

# ISS-20260520T035443109Z-SELFHOST-PARSER-REPORT-CONTRACT-EXPE-2F0D52E2: selfhost parser report contract expects stale assertion count

## 概要

The parser report source policy still expects 21 std/test assertions, while tests/stdlib/neplg2_parser.n.md now pins 22 assertion rows in stdout. The doctest metadata is correct, but the policy fails and would make source-policy runs noisy.

## 対象

- `nodesrc/test_selfhost_parser_report_contract.js, tests/stdlib/neplg2_parser.n.md`

## 根拠

- `tests/stdlib/neplg2_parser.n.md` の stdout fixture は `Checked [...]` と `[0]` から `[21]` までの 22 件の assertion report を固定している。
- `nodesrc/test_selfhost_parser_report_contract.js` は `expectedCheckCounts = [21]` のままだったため、実際の doctest metadata と source policy の期待値がずれていた。
- これは parser 実装や doctest metadata の問題ではなく、report contract policy 側の stale expected count である。

## 問題

The parser report source policy still expects 21 std/test assertions, while tests/stdlib/neplg2_parser.n.md now pins 22 assertion rows in stdout. The doctest metadata is correct, but the policy fails and would make source-policy runs noisy.

## 影響

A stale policy makes CI/source-policy feedback unreliable and can hide real parser report regressions behind an obsolete expected count.

## 修正方針

Update the parser report contract to derive or explicitly pin the current assertion count, document the regression in an issue, and verify the focused parser doctest plus the policy.

## 検証

Run node nodesrc/test_selfhost_parser_report_contract.js, node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/agent1-parser-report-contract.json -j 1 --dist web/dist --assert-io, node nodesrc/issues.js check, and git diff --check.

## 対応結果

- `nodesrc/test_selfhost_parser_report_contract.js` の `expectedCheckCounts` を現在の stdout fixture と一致する 22 件へ更新した。
- parser doctest 側の stdout / `exit_code: 0` / `stdio, normalize_newlines` metadata は既に正しいため変更していない。
- policy は `ret:` 禁止、stdout report 固定、`checks_print_report` から `checks_exit_code` への順序を引き続き確認する。

## 検証結果

- `node nodesrc/test_selfhost_parser_report_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/agent1-parser-report-contract.json -j 1 --dist web/dist --assert-io`: 1/1 passed
