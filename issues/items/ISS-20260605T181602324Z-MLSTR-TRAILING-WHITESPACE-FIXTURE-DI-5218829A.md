---
id: ISS-20260605T181602324Z-MLSTR-TRAILING-WHITESPACE-FIXTURE-DI-5218829A
title: "mlstr trailing whitespace fixture disagrees with current output"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-06-05
updated: 2026-06-05
target: "tests/stdlib/string.n.md, parser mlstr handling"
---

# ISS-20260605T181602324Z-MLSTR-TRAILING-WHITESPACE-FIXTURE-DI-5218829A: mlstr trailing whitespace fixture disagrees with current output

## 概要

tests/stdlib/string.n.md::test_mlstr_trailing_whitespace expects mlstr: to preserve three trailing spaces after line1, but the current run prints [line1\nline2]END. This means either the fixture encodes a stale contract or the parser/runtime currently trims mlstr trailing whitespace contrary to the documented expectation.

## 対象

- `tests/stdlib/string.n.md, parser mlstr handling`

## 根拠

- 未記入

## 問題

tests/stdlib/string.n.md::test_mlstr_trailing_whitespace expects mlstr: to preserve three trailing spaces after line1, but the current run prints [line1\nline2]END. This means either the fixture encodes a stale contract or the parser/runtime currently trims mlstr trailing whitespace contrary to the documented expectation.

## 影響

Broad string regression runs cannot be treated as fully green, and the mlstr whitespace contract is ambiguous for documentation, examples, and source text fixtures that need exact byte preservation.

## 修正方針

Confirm the intended mlstr contract from the parser/spec, then either preserve trailing whitespace in mlstr body lines or update the test and documentation to state that body line trailing whitespace is trimmed. Keep the choice as a typed parser/source-text contract rather than a test-only expectation.

## 検証

node nodesrc/tests.js -i tests/stdlib/string.n.md --no-tree -o tmp/mlstr-trailing-whitespace-contract.json -j 1 --dist web/dist --assert-io
