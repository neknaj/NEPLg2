---
id: ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941
title: "selfhost parser and checker do not implement full prefix expression and type range contracts"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl"
---

# ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941: selfhost parser and checker do not implement full prefix expression and type range contracts

## 概要

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 影響

Selfhost compiler modules cannot validate the same prefix argument and % type ranges that the Rust compiler and Web highlighting now rely on, so stdlib/neplg2 tests can pass smoke cases while missing core language invariants.

## 修正方針

Implement or stage a real PrefixList/TypePrefixList parser boundary, connect checker range validation, and keep partial parser smoke paths marked as transitional rather than public compiler contract.

## 検証

Add normal tests for prefix argument extent, %TypeExpr extent, nested block arguments, malformed prefix calls, and checker diagnostics once cfg-test-style tests are available.
