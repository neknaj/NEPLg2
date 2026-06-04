---
id: ISS-20260604T034256176Z-SELFHOST-PARSER-CAN-ACCEPT-UNCLOSED--C0703412
title: "selfhost parser can accept unclosed raw blocks and excess dedent as Ok"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl, stdlib/neplg2/core/syntax/parser/module_parser/state.nepl"
---

# ISS-20260604T034256176Z-SELFHOST-PARSER-CAN-ACCEPT-UNCLOSED--C0703412: selfhost parser can accept unclosed raw blocks and excess dedent as Ok

## 概要

Subagent audit found raw/offside parser state where active raw EOF can return Ok ast and dedent depth can saturate to 0. plan.md states indentation errors are errors, and Zenn guidance requires invalid states to be explicit Result/enum outcomes.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl, stdlib/neplg2/core/syntax/parser/module_parser/state.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found raw/offside parser state where active raw EOF can return Ok ast and dedent depth can saturate to 0. plan.md states indentation errors are errors, and Zenn guidance requires invalid states to be explicit Result/enum outcomes.

## 影響

Malformed source can be accepted by selfhost parser paths, weakening parser diagnostics and any tooling that depends on selfhost parsing.

## 修正方針

Add diagnostic variants such as InvalidDedent and UnclosedRawBlock, make parser state transitions return Result, and remove saturating dedent for invalid input.

## 検証

Add regular tests for unclosed raw block EOF, extra dedent, valid nested block, and recovery diagnostics through source and hand-built token streams.
