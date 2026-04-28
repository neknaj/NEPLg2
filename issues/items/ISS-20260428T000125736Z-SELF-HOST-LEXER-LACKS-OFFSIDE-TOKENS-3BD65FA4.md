---
id: ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4
title: "self-host lexer lacks offside tokens and Rust parity fixtures"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/**, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260428T000125736Z-SELF-HOST-LEXER-LACKS-OFFSIDE-TOKENS-3BD65FA4: self-host lexer lacks offside tokens and Rust parity fixtures

## 概要

stdlib/neplg2 の lexer は whitespace/comment/identifier/int/string/punctuation/EOF の byte lexer まで進んでいるが、#indent と offside rule による Indent/Dedent token をまだ生成せず、Rust lexer の token JSON と比較する parity fixture もない。

## 対象

- `stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/**, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- `stdlib/neplg2/core/syntax/token.nepl` の `TokenKind` は `Newline` までで、`Indent` / `Dedent` を持たない。
- `stdlib/neplg2/core/syntax/lexer.nepl` は horizontal whitespace と newline を個別 token として扱うだけで、indent stack や `#indent` directive による offside rule を持たない。
- `stdlib/neplg2/README.md` は indent / dedent と Rust lexer JSON parity を後続 issue と記述しているが、open issue としてはまだ分離されていなかった。

## 問題

stdlib/neplg2 の lexer は whitespace/comment/identifier/int/string/punctuation/EOF の byte lexer まで進んでいるが、#indent と offside rule による Indent/Dedent token をまだ生成せず、Rust lexer の token JSON と比較する parity fixture もない。

## 影響

S1 の成功条件である lexer/parser parity に到達できず、parser 実装を開始しても実際の NEPLg2 source の block 構造を安定して読めない。後続の AST JSON 比較も token 境界の不一致に引きずられる。

## 修正方針

TokenKind に Indent/Dedent と directive に必要な token を追加し、Rust lexer と同じ offside rule を self-host lexer に移植する。Rust 側 tree fixture と同じ入力で token JSON を比較する focused test を追加し、その上で parser module を進める。

## 検証

stdlib/neplg2 lexer focused doctest、Rust lexer JSON parity fixture、S1 parser smoke を実行し、indent/dedent を含む関数・if・match・import source が同じ token stream になることを確認する。
