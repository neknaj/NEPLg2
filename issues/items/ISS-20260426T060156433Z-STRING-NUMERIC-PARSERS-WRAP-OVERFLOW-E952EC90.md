---
id: ISS-20260426T060156433Z-STRING-NUMERIC-PARSERS-WRAP-OVERFLOW-E952EC90
title: "string numeric parsers wrap overflow instead of returning Err"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/string.nepl
---

# ISS-20260426T060156433Z-STRING-NUMERIC-PARSERS-WRAP-OVERFLOW-E952EC90: string numeric parsers wrap overflow instead of returning Err

## 概要

stdlib/alloc/string.nepl の to_u128_radix と to_i128_radix はコメントで overflow を mod 2^128 wrap と明記しており、to_i64_radix は to_i128_radix の結果を range check せず i64 へ cast している。to_i32_radix も to_i64_radix 経由なので、巨大入力が範囲外 Err ではなく wrap 後の値として受理され得る。

## 対象

- `stdlib/alloc/string.nepl`

## 根拠

- 未記入

## 問題

stdlib/alloc/string.nepl の to_u128_radix と to_i128_radix はコメントで overflow を mod 2^128 wrap と明記しており、to_i64_radix は to_i128_radix の結果を range check せず i64 へ cast している。to_i32_radix も to_i64_radix 経由なので、巨大入力が範囲外 Err ではなく wrap 後の値として受理され得る。

## 影響

CLI option、lexer、parser、競技入力などで範囲外数値を正常値として扱い、diagnostic や実行結果が入力に対して不正になる。stdlib の Result ベース入力処理方針とも矛盾する。

## 修正方針

radix ごとの乗算前/加算前 overflow check を実装し、u128/i128/i64/i32 の各 parser が範囲外を Result::Err で返すように統一する。境界値、境界外、符号付き最小値を doctest に追加する。

## 検証

string numeric parser doctest に i32/i64/i128/u128 の境界値と境界外入力を追加し、stdlib 全体テストを通す。
