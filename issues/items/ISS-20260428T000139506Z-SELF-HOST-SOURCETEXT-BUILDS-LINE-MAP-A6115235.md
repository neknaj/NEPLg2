---
id: ISS-20260428T000139506Z-SELF-HOST-SOURCETEXT-BUILDS-LINE-MAP-A6115235
title: "self-host SourceText builds line maps with per-byte recursion"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md"
---

# ISS-20260428T000139506Z-SELF-HOST-SOURCETEXT-BUILDS-LINE-MAP-A6115235: self-host SourceText builds line maps with per-byte recursion

## 概要

source_text_collect_line_starts は source の各 byte ごとに再帰し、LF/CR/CRLF を見つけるたびに同じ関数を呼び直す。大きな stdlib file や将来の generated source を読むと、line map 構築だけで call stack を byte 長に比例して消費する。

## 対象

- `stdlib/neplg2/core/infra/text.nepl, tests/stdlib/neplg2_text.n.md`

## 根拠

- `stdlib/neplg2/core/infra/text.nepl` の `source_text_collect_line_starts` は `idx` を 1 byte または CRLF 2 byte 進めるたびに自分自身を呼び直す。
- `doc/neplg2/self_host_plan.md` の S4 は deep traversal を explicit stack で実装する方針を置いており、source text でも入力長比例の call stack 消費は避けるべきである。
- 現在の stdlib には `stdlib/core/math.nepl` 4435 lines、`stdlib/alloc/string.nepl` 2479 lines など大きな入力が既にあり、self-host compiler はこれらを継続的に読む。

## 問題

source_text_collect_line_starts は source の各 byte ごとに再帰し、LF/CR/CRLF を見つけるたびに同じ関数を呼び直す。大きな stdlib file や将来の generated source を読むと、line map 構築だけで call stack を byte 長に比例して消費する。

## 影響

self-host compiler が stdlib/core/math.nepl や alloc/string.nepl 程度の入力を継続的に扱う段階で、診断ではなく stack overflow / trap に到達する可能性がある。S4 で deep HIR traversal を explicit stack にする方針とも矛盾する。

## 修正方針

source_text_collect_line_starts を index 付き while/loop 相当の反復 helper に置き換える。CRLF の 2 byte advance と allocation failure の Result 伝播は維持し、line start table の契約を変えない。

## 検証

長い 1 行、数万行、CRLF 混在の large source fixture を追加し、source_text_new と offset lookup が panic/trap せず O(n) で完走することを stdlib/neplg2_text focused test で確認する。
