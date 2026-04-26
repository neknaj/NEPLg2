---
id: ISS-20260426T060156433Z-STRING-NUMERIC-PARSERS-WRAP-OVERFLOW-E952EC90
title: "string numeric parsers wrap overflow instead of returning Err"
area: stdlib
status: verified
resolved: true
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

- `to_u128_radix` は `res = res * radix + digit` を `u128_mul_small` / `u128_add_small` で直接行い、`u128` の mod 2^128 ラップを検出していなかった。
- `to_i128_radix` は `to_u128_radix` の magnitude を range check せず `i128` bitcast していたため、正の範囲外値を負値として受理し得た。
- `to_i64_radix` は `i128` から `i64` への縮小前に符号拡張一致を確認しておらず、範囲外値を切り詰め得た。

## 問題

stdlib/alloc/string.nepl の to_u128_radix と to_i128_radix はコメントで overflow を mod 2^128 wrap と明記しており、to_i64_radix は to_i128_radix の結果を range check せず i64 へ cast している。to_i32_radix も to_i64_radix 経由なので、巨大入力が範囲外 Err ではなく wrap 後の値として受理され得る。

## 影響

CLI option、lexer、parser、競技入力などで範囲外数値を正常値として扱い、diagnostic や実行結果が入力に対して不正になる。stdlib の Result ベース入力処理方針とも矛盾する。

## 修正方針

radix ごとの乗算前/加算前 overflow check を実装し、u128/i128/i64/i32 の各 parser が範囲外を Result::Err で返すように統一する。境界値、境界外、符号付き最小値を doctest に追加する。

## 検証

string numeric parser doctest に i32/i64/i128/u128 の境界値と境界外入力を追加し、stdlib 全体テストを通す。

## 対応結果

- `u128_max / radix` の商と余りを使う `u128_can_mul_add_small` を追加し、`to_u128_radix` が更新前に overflow を判定するようにした。
- `to_i128_radix` に正負別の magnitude range check を追加し、正の最大値、負の最小値、その外側を区別するようにした。
- `to_i64_radix` に i128 の上位 word と i64 の符号拡張が一致するかの検査を追加した。
- `tests/stdlib/string_numeric_overflow.n.md` を追加し、u128/i128/i64/i32 の境界値と境界外入力を固定した。
- 作業中に `Result<i64,i32>` の `Result::Ok _` wildcard arm が invalid wasm を生成する別問題を確認したため、`ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655` を追加し、Discord へ報告した。

## 確認結果

- `node nodesrc/tests.js -i tests/stdlib/string_numeric_overflow.n.md --no-tree -o tmp/string-numeric-overflow-after-str-eq.json -j 1`: `total=8`, `passed=8`, `failed=0`
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string.n.md -i tests/stdlib/stdlib.n.md -i tests/stdlib/string_numeric_overflow.n.md --no-tree -o tmp/string-numeric-overflow-suite.json -j 2`: `total=57`, `passed=57`, `failed=0`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/string-numeric-overflow-stdlib-full.json -j 4`: `total=404`, `passed=404`, `failed=0`
