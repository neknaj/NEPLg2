---
id: ISS-20260428T192326177Z-SELF-HOST-LEXER-KEYWORD-CLASSIFIER-R-67015A00
title: "self-host lexer keyword classifier remains nested if decision tree"
area: selfhost
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/lexer.nepl; nodesrc/test_stdlib_match_decision_trees.js"
---

# ISS-20260428T192326177Z-SELF-HOST-LEXER-KEYWORD-CLASSIFIER-R-67015A00: self-host lexer keyword classifier remains nested if decision tree

## 概要

The self-host lexer function lex_keyword_kind still encodes the fixed keyword table as a deep if/else-if chain. This was the deepest HIR shape found while investigating the import_spec wasm codegen stack issue, and it was not covered by the existing stdlib match-decision-tree regression.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl; nodesrc/test_stdlib_match_decision_trees.js`

## 根拠

- 未記入

## 問題

The self-host lexer function lex_keyword_kind still encodes the fixed keyword table as a deep if/else-if chain. This was the deepest HIR shape found while investigating the import_spec wasm codegen stack issue, and it was not covered by the existing stdlib match-decision-tree regression.

## 影響

Keyword additions are hard to review as a table, the code remains inconsistent with the match-first stdlib policy, and compiler stack pressure can reappear in similar finite classifiers even though the wasm codegen path now lowers else-if chains iteratively.

## 修正方針

Replace lex_keyword_kind with a table-like classifier that uses match where the language can express it, or introduce a small normalized key enum before matching if direct str match is still unavailable. Extend the static match-decision-tree regression to cover self-host lexer keyword classification.

## 検証

node nodesrc/test_stdlib_match_decision_trees.js; node nodesrc/tests.js -i stdlib/neplg2/core/syntax/lexer.nepl -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/selfhost-lexer-keyword-match.json -j 1
