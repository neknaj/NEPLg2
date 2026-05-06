---
id: ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D
title: "Self-host lexer lex_next timeout blocks parser loader and module graph doctests"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/graph.nepl"
---

# ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D: Self-host lexer lex_next timeout blocks parser loader and module graph doctests

## 概要

On current main, even an empty lex_all_with_file_id smoke case times out at the default 60000ms wasm test budget. The graph, loader, and module_parser doctests also time out because they all enter lexer lex_next/lex_all. The stdlib_map timeout was separately reduced to a compile-time owner-leak diagnostic and then fixed, so the remaining graph timeout is rooted in lexer/static-check complexity rather than path mapping.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/graph.nepl`

## 根拠

- 未記入

## 問題

On current main, even an empty lex_all_with_file_id smoke case times out at the default 60000ms wasm test budget. The graph, loader, and module_parser doctests also time out because they all enter lexer lex_next/lex_all. The stdlib_map timeout was separately reduced to a compile-time owner-leak diagnostic and then fixed, so the remaining graph timeout is rooted in lexer/static-check complexity rather than path mapping.

## 影響

Self-host parser/loader/import-graph doctests cannot provide CI signal, and graph work cannot verify import traversal while lex_next stays above the default per-case budget. The problem also hides whether graph DFS itself is correct.

## 修正方針

Redesign lexer tokenization so lex_next does not force static checking/codegen through a monolithic resource-bearing branch tree. Prefer Copy-only token range classification, avoid temporary str owner creation while classifying identifiers/directives, and split directive/keyword/token construction so enum/match coverage remains explicit without concentrating all branches in one owner-returning function.

## 検証

Run node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_fix.json -j 1, then stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, and stdlib/neplg2/core/module/graph.nepl focused doctests under the default 60000ms timeout.
