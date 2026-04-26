---
id: ISS-20260426T023700894Z-TRAITS-TEXT-DOCTEST-RUNTIME-RETURN-M-D1631318
title: "traits_text doctest が runtime return mismatch になる"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "tests/stdlib/traits_text.n.md, stdlib/core/traits, stdlib/std/string.nepl"
---

# ISS-20260426T023700894Z-TRAITS-TEXT-DOCTEST-RUNTIME-RETURN-M-D1631318: traits_text doctest が runtime return mismatch になる

## 概要

tests/stdlib/traits_text.n.md::doctest#1 が expected 14 に対して actual 131074 を返す。

## 対象

- `tests/stdlib/traits_text.n.md, stdlib/core/traits, stdlib/std/string.nepl`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/rv-stdlib-018-final-tests-stdlib-crlf.json -j 4` で `tests/stdlib/traits_text.n.md::doctest#1` が runtime failure になった。
- failure は `return value mismatch` で、期待値は `14`、実際の戻り値は `131074`。
- 同じ広域検証で streamio は 13/13 green になっているため、streamio 修正とは別原因として分離する。

## 問題

tests/stdlib/traits_text.n.md::doctest#1 が expected 14 に対して actual 131074 を返す。

## 影響

文字列関連 trait の doctest が信頼できず、Clone / text conversion / output helper の組み合わせで値表現が崩れている可能性を検出できない。

## 修正方針

期待値 14 の意味を分解し、どの式が 131074 を返しているか最小化する。fixture ずれなら期待値を根拠付きで更新し、runtime 表現の混入なら trait 実装または string helper を修正する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/traits_text.n.md --no-tree -o tmp/traits-text-issue.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/traits-text-tests-stdlib.json -j 4`
