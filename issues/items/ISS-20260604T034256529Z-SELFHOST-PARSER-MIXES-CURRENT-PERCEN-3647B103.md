---
id: ISS-20260604T034256529Z-SELFHOST-PARSER-MIXES-CURRENT-PERCEN-3647B103
title: "selfhost parser mixes current percent type syntax with legacy paren and angle syntax boundaries"
area: selfhost
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl, stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl"
---

# ISS-20260604T034256529Z-SELFHOST-PARSER-MIXES-CURRENT-PERCEN-3647B103: selfhost parser mixes current percent type syntax with legacy paren and angle syntax boundaries

## 概要

Subagent audit found LParen still classified as expression start and declaration parsing still oriented around legacy angle/generic paths while Percent is only partially connected. This conflicts with the current NEPLg2.1 prefix type syntax and the user requirement that parentheses are not used in NEPLg2 source.

## 対象

- `stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl, stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found LParen still classified as expression start and declaration parsing still oriented around legacy angle/generic paths while Percent is only partially connected. This conflicts with the current NEPLg2.1 prefix type syntax and the user requirement that parentheses are not used in NEPLg2 source.

## 影響

Parser and syntax tooling can disagree about whether % type annotations, legacy angle forms, and parentheses belong to current source, producing stale examples or incorrect range diagnostics.

## 修正方針

Unify token predicates and declaration parsing around current % type syntax, move legacy forms to explicit compatibility or compile-fail diagnostics, and document migration boundaries.

## 検証

Add regular tests for % type annotation ranges, function type ranges, legacy angle rejection, LParen rejection in current source, and diagnostic spans.
