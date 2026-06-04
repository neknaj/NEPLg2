---
id: ISS-20260604T034257267Z-NM-JSON-AND-HTML-GENERATION-DUPLICAT-C449CDF2
title: "nm JSON and HTML generation duplicate block traversal instead of sharing a document event stream"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/nm/parser/document.nepl, stdlib/nm/html_gen.nepl"
---

# ISS-20260604T034257267Z-NM-JSON-AND-HTML-GENERATION-DUPLICAT-C449CDF2: nm JSON and HTML generation duplicate block traversal instead of sharing a document event stream

## 概要

Subagent audit found JSON parsing and HTML generation walking similar block structures independently. Zenn guidance prefers shared abstractions and DAG-shaped responsibilities when they remove real duplication without adding runtime cost.

## 対象

- `stdlib/nm/parser/document.nepl, stdlib/nm/html_gen.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found JSON parsing and HTML generation walking similar block structures independently. Zenn guidance prefers shared abstractions and DAG-shaped responsibilities when they remove real duplication without adding runtime cost.

## 影響

Malformed fences, heading ranges, and block boundary behavior can diverge between JSON and HTML outputs, making documentation tooling harder to test.

## 修正方針

Introduce a range-based document block event stream shared by JSON and HTML serializers, keeping serializers responsible only for output format.

## 検証

Add semantic snapshot tests for the same NM input rendered to JSON and HTML, plus malformed fence and heading boundary cases.
