---
id: ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B
title: "self-host lexer needs full Rust token JSON parity harness"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/analyze_source.js"
---

# ISS-20260428T084929443Z-SELF-HOST-LEXER-NEEDS-FULL-RUST-TOKE-E365D38B: self-host lexer needs full Rust token JSON parity harness

## 概要

Indent/Dedent と #indent 幅更新は self-host lexer に入ったが、Rust lexer の analyze_lex JSON と完全比較するには keywords、directive token、doc/raw/mlstr token、newline span などを含む token model parity と比較 harness がまだ不足している。

## 対象

- `stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/analyze_source.js`

## 根拠

- `ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4` で offside `Indent` / `Dedent` と `#indent` 幅更新は追加した。
- ただし現行 self-host lexer は `fn` などの keyword を `Identifier` として扱う段階で、Rust lexer の `KwFn` などとは token kind が一致しない。
- `#entry` / `#target` / `#import` / doc comment / raw wasm/llvm / mlstr など、Rust lexer JSON と比較するには追加 token kind と normalized comparison harness が必要。

## 問題

Indent/Dedent と #indent 幅更新は self-host lexer に入ったが、Rust lexer の analyze_lex JSON と完全比較するには keywords、directive token、doc/raw/mlstr token、newline span などを含む token model parity と比較 harness がまだ不足している。

## 影響

S1 parser parity を確認する段階で token kind と span の差分が混ざり、parser 側の不一致と lexer 側の不一致を切り分けにくい。

## 修正方針

self-host TokenKind を Rust lexer token set へ段階的に合わせ、Rust analyze_lex JSON と self-host lexer output を同一 fixture で比較する nodesrc harness を追加する。offside token の focused doctest は維持し、full parity は別 issue として進める。

## 検証

Rust analyze_lex JSON と self-host lexer output の normalized token stream 比較を、#entry/#target/#import/#indent、keyword、literal、doc/raw/mlstr、nested block source で実行する。
