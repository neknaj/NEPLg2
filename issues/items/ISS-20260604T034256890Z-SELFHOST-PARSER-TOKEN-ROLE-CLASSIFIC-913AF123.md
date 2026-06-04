---
id: ISS-20260604T034256890Z-SELFHOST-PARSER-TOKEN-ROLE-CLASSIFIC-913AF123
title: "selfhost parser token role classification is duplicated across large match tables"
area: selfhost
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl, stdlib/neplg2/core/syntax/parser/module_parser/item_kind.nepl"
---

# ISS-20260604T034256890Z-SELFHOST-PARSER-TOKEN-ROLE-CLASSIFIC-913AF123: selfhost parser token role classification is duplicated across large match tables

## 概要

Subagent audit found visibility, declaration head kind, and item kind classification distributed across separate match tables. This conflicts with the Zenn guidance around DAG, directory hierarchy, and static verification of classification responsibilities.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl, stdlib/neplg2/core/syntax/parser/module_parser/item_kind.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found visibility, declaration head kind, and item kind classification distributed across separate match tables. This conflicts with the Zenn guidance around DAG, directory hierarchy, and static verification of classification responsibilities.

## 影響

Adding or migrating syntax tokens requires editing several tables, making it easy for parser, syntax highlight, and diagnostic behavior to drift.

## 修正方針

Introduce a token role enum/helper table and route visibility/head/item classification through the shared contract, with source policy guarding duplicated classification.

## 検証

Add table-diff regular tests for token role classification and current syntax fixtures.
