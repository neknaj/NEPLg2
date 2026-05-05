---
id: ISS-20260505T040130265Z-STDIO-ANSI-API-ENUM-MATCH-BDF2FB5E
title: "stdio ANSI 色 API を enum/match で型付けして整理する"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: stdlib/std/stdio/ansi.nepl
---

# ISS-20260505T040130265Z-STDIO-ANSI-API-ENUM-MATCH-BDF2FB5E: stdio ANSI 色 API を enum/match で型付けして整理する

## 概要

std/stdio/ansi は root から分離されたが、色や style を個別の文字列返却関数として管理しており、呼び出し側が任意 str を渡せるため静的検査が効きにくい。module も 486 lines あり、同型の doc/function が反復している。

## 対象

- `stdlib/std/stdio/ansi.nepl`

## 根拠

- 未記入

## 問題

std/stdio/ansi は root から分離されたが、色や style を個別の文字列返却関数として管理しており、呼び出し側が任意 str を渡せるため静的検査が効きにくい。module も 486 lines あり、同型の doc/function が反復している。

## 影響

ANSI 色指定の typo や unsupported code が型で表現されず、selfhost/stdlib の方針である enum と match による網羅性検査を活かせない。今後の色追加時に重複実装が増える。

## 修正方針

AnsiStyle / AnsiColor などの enum を設計し、escape code 生成を match に集約する。print_color 系は enum を受け取る typed API に改め、必要なら互換 facade は段階的に削除する。doc と regression で root facade への逆流と網羅的 match を固定する。

## 検証

stdio ansi doctest、stdout 回帰、source policy で enum/match 境界と root 逆流防止を確認する。
