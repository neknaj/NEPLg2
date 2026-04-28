---
id: ISS-20260428T192326177Z-SELF-HOST-LEXER-KEYWORD-CLASSIFIER-R-67015A00
title: "self-host lexer keyword classifier remains nested if decision tree"
area: selfhost
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-04-28
updated: 2026-04-29
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

`lex_keyword_kind` を length bucket + scalar key `match` の table-like classifier に置き換えました。NEPLg2 の `match` は現時点で `str` pattern を直接扱わないため、まず `string::len lexeme` で候補集合を絞り、各 bucket で先頭 2 byte の key を `match` します。各 arm は `lex_keyword_kind_if_eq` により実文字列を検証するため、同じ length/prefix の非 keyword を誤分類しません。

単一の巨大な decision tree ではなく、keyword の byte length ごとに小さな table として分割しました。これにより lexer の見通しを保ちつつ、compiler 側の block-wrapped else-if stack 問題にも依存しない形にしています。static regression は `lex_keyword_kind` 本体と各 bucket helper を対象にし、`if` decision tree への回帰を検出します。

## 検証

- `node nodesrc\test_stdlib_match_decision_trees.js`: pass
- `trunk build`: pass（先行して `ISS-20260428T193442835Z...` の wasm codegen 修正を反映）
- `node nodesrc\tests.js -i stdlib\neplg2\core\syntax\lexer.nepl -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\selfhost-lexer-keyword-match.json -j 1`: total=13 passed=13
