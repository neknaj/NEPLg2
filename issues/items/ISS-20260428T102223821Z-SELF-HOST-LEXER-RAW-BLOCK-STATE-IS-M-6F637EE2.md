---
id: ISS-20260428T102223821Z-SELF-HOST-LEXER-RAW-BLOCK-STATE-IS-M-6F637EE2
title: "self-host lexer raw block state is missing"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260428T102223821Z-SELF-HOST-LEXER-RAW-BLOCK-STATE-IS-M-6F637EE2: self-host lexer raw block state is missing

## 概要

TokenKind に WasmText / LlvmIrText は追加されたが、self-host lexer の lex_all_loop は #wasm: / #llvmir: 後の raw block state を保持しないため、raw block 本文を Rust lexer と同じ WasmText / LlvmIrText token stream として生成できない。

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- 未記入

## 問題

TokenKind に WasmText / LlvmIrText は追加されたが、self-host lexer の lex_all_loop は #wasm: / #llvmir: 後の raw block state を保持しないため、raw block 本文を Rust lexer と同じ WasmText / LlvmIrText token stream として生成できない。

## 影響

self-host parser parity で raw wasm / llvmir block の lexer 差分が parser 差分に混ざり、S1 parser 実装時に raw block directive の扱いを切り分けにくい。

## 修正方針

#wasm: / #llvmir: を読んだ行で pending raw mode と expected base indent を保持し、次行以降は base indent を満たす間 WasmText / LlvmIrText を生成する。dedent で raw mode を終了し、Rust lexer と同じ newline / indent / dedent span を維持する。

## 検証

Rust analyze_lex JSON と self-host lexer output を #wasm: / #llvmir: fixture で比較し、WasmText / LlvmIrText / Indent / Dedent / Newline の kind と span が一致することを確認する。
