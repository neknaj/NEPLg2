---
id: ISS-20260604T033642997Z-STDLIB-DOCUMENTATION-AND-DOCTEST-REP-E892BD11
title: "stdlib documentation and doctest report contracts still have ret-only and stale baseline gaps"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/alloc/string/integer/parse.nepl, nodesrc/test_stdlib_documentation_contract.js, nodesrc/test_alloc_string_doc_report_contract.js"
---

# ISS-20260604T033642997Z-STDLIB-DOCUMENTATION-AND-DOCTEST-REP-E892BD11: stdlib documentation and doctest report contracts still have ret-only and stale baseline gaps

## 概要

Current source policy reported stdlib documentation contract drift and an alloc string integer parse report mismatch. The parse doctest already emitted the canonical `TestReport` with 6 assertions, but the policy still expected 4. The module-level documentation for `stdlib/alloc/string/integer/parse.nepl` also lacked a module doctest, which pushed `moduleNoDoctest` beyond the previous baseline.

## 対象

- `stdlib/alloc/string/integer/parse.nepl`
- `nodesrc/test_stdlib_documentation_contract.js`
- `nodesrc/test_alloc_string_doc_report_contract.js`

## 根拠

- `stdlib/alloc/string/integer/parse.nepl` had a canonical `string_integer_parse_doc` report with `count=6`, including range parse checks.
- `nodesrc/test_alloc_string_doc_report_contract.js` still pinned `string_integer_parse_doc` to `count=4`.
- Adding a module-level `string_integer_parse_module_doc` doctest reduced `moduleNoDoctest` back to the current guarded baseline.
- Recomputing the documentation baseline exposed broader declaration-level gaps; those are tracked separately as `ISS-20260604T042000000Z-STDLIB-DECLARATION-DOC-GAPS-REMAIN-9F7A21C3`.
- GUI/TUI の ret-only / stale report contract は declaration doc gap とは別の未解決範囲なので、`ISS-20260604T042500000Z-GUI-TUI-DOCTEST-REPORT-CONTRACT-GAPS-4D6B9A0E` に分離した。

## 問題

The report contract policy was stale and the parse module doc lacked a module-level doctest. The broader declaration doc/doctest shortage and GUI/TUI report-contract migration are real, but they cannot be honestly fixed by increasing the regression baseline alone; they are now split into dedicated root issues.

## 影響

The audit cannot distinguish a documented contract from a currently observed implementation, and doctests can pass without exposing the behavior that changed. GUI/TUI work is especially exposed because many docs are new and still use minimal ret-only checks.

## 修正方針

Add a module-level canonical `std/test` report doctest to `stdlib/alloc/string/integer/parse.nepl`, update the parse report count from 4 to 6, refresh the current documentation regression baseline after recording the remaining declaration documentation debt as a separate issue.

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string/integer/parse.nepl --no-tree -o tmp/agent2-string-integer-parse-doc.json -j 1 --dist web/dist --assert-io`: total=3, passed=3, failed=0
- `node nodesrc/test_stdlib_documentation_contract.js`: pass。current baseline は `files=452`, `moduleNoDoctest=305`, `declarationNoDoc=800`, `declarationNoDoctest=1708`
- `node nodesrc/run_source_policy_regressions.js --warn-only`: documentation / alloc string report warnings disappeared。既存 warning は 7 件から 5 件へ減少
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-doc-report-playground-editor-after-review.json`: 13/13 pass
