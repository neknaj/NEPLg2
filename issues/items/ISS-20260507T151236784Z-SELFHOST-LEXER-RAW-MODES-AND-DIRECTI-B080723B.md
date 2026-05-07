---
id: ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B
title: "Selfhost lexer raw modes and directives bypass enum match coverage"
area: selfhost
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/test_selfhost_lexer_rust_parity.js"
---

# ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B: Selfhost lexer raw modes and directives bypass enum match coverage

## 概要

The selfhost lexer stores raw block state as i32 sentinels: raw_mode and pending_raw_mode use 0/1/2, lex_token_pending_raw_mode returns numeric mode values, and lex_raw_kind maps non-1 values to LlvmIrText. Directive classification is also a deep nested if chain instead of a finite enum or hash+match classifier.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/test_selfhost_lexer_rust_parity.js`

## 根拠

- 未記入

## 問題

The selfhost lexer stores raw block state as i32 sentinels: raw_mode and pending_raw_mode use 0/1/2, lex_token_pending_raw_mode returns numeric mode values, and lex_raw_kind maps non-1 values to LlvmIrText. Directive classification is also a deep nested if chain instead of a finite enum or hash+match classifier.

## 影響

Raw block handling and directive additions are memory/type-safety-adjacent parser inputs. With numeric modes, the static checker cannot enforce exhaustive handling when new raw modes or directive kinds are added, and an unexpected numeric value silently becomes LlvmIrText. The directive chain also repeats the pattern previously rejected for keyword classification.

## 修正方針

Introduce a SelfhostLexerRawMode enum and, if needed, separate Pending/Active raw state records so raw mode transitions are match-exhaustive. Replace directive classification with a hash/key + match + str_starts_with_at verification table following the keyword and CLI arg classifier pattern. Keep #indent/#wasm/#llvmir parity with Rust lexer.

## 検証

Add source-policy tests rejecting i32 raw_mode/pending_raw_mode state and lex_raw_kind fallback behavior, plus regression fixtures for #wasm, #llvmir, #indent, #if[target], unknown directives, and raw block dedent. Keep rust/selfhost lexer parity checks covering directive and raw text token kinds/spans.
