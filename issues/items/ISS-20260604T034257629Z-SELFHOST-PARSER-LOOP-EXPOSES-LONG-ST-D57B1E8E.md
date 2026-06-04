---
id: ISS-20260604T034257629Z-SELFHOST-PARSER-LOOP-EXPOSES-LONG-ST-D57B1E8E
title: "selfhost parser loop exposes long state threading instead of ParserState transitions"
area: selfhost
status: open
resolved: false
priority: P3
type: maintenance
created: 2026-06-04
updated: 2026-06-04
target: stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl
---

# ISS-20260604T034257629Z-SELFHOST-PARSER-LOOP-EXPOSES-LONG-ST-D57B1E8E: selfhost parser loop exposes long state threading instead of ParserState transitions

## 概要

Subagent audit found parser loop helpers threading source, tokens, n, idx, depth, mode, and ast through long signatures. This weakens the Zenn guidance around shallow structure, immutable state transitions, and clear responsibility boundaries.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found parser loop helpers threading source, tokens, n, idx, depth, mode, and ast through long signatures. This weakens the Zenn guidance around shallow structure, immutable state transitions, and clear responsibility boundaries.

## 影響

Parser invariants are spread across call sites, making offside/raw-mode bugs harder to localize and test.

## 修正方針

Introduce ParserInput, ParserState, and ParserStep-style functions so each token transition is State + Token -> Result State Diagnostic.

## 検証

Add regular tests for parser state transition units, raw-mode state, dedent state, and AST append behavior.
