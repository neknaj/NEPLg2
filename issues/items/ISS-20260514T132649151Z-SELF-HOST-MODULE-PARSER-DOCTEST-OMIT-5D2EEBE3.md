---
id: ISS-20260514T132649151Z-SELF-HOST-MODULE-PARSER-DOCTEST-OMIT-5D2EEBE3
title: "Self-host module_parser doctest omits current math import for eq"
area: selfhost
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-14
updated: 2026-05-14
target: stdlib/neplg2/core/syntax/parser/module_parser.nepl
---

# ISS-20260514T132649151Z-SELF-HOST-MODULE-PARSER-DOCTEST-OMIT-5D2EEBE3: Self-host module_parser doctest omits current math import for eq

## 概要

Focused verification after the ResourceIR summary worklist timeout fix reaches module_parser doctest compile phase, but the embedded doctest calls eq without importing the current math/comparison API, so it fails with resolve.identifier.undefined before parser behavior is exercised.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl`

## 根拠

- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree --dist web/dist -o tmp/agent1_module_parser_summary_worklist_default.json -j 1 --assert-io`: total=1, failed=1。compile phase で `/virtual/entry.nepl:11:16` の `eq` が `resolve.identifier.undefined` になり、timeout ではなく stale doctest import として切り分けた。

## 問題

Focused verification after the ResourceIR summary worklist timeout fix reaches module_parser doctest compile phase, but the embedded doctest calls eq without importing the current math/comparison API, so it fails with resolve.identifier.undefined before parser behavior is exercised.

## 影響

module_parser cannot be used as a focused regression for parser/loader progress because the doctest stops on a stale fixture import instead of validating the self-host parser smoke path.

## 修正方針

Update the module_parser doctest snippet to import the API that provides eq under the current stdlib module layout, and keep the example aligned with the documented stdlib doctest import policy.

## 検証

Run NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree --dist web/dist -o tmp/module_parser_doctest_after_import_fix.json -j 1 --assert-io.
