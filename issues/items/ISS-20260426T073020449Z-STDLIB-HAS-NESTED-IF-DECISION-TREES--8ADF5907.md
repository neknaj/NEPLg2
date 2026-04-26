---
id: ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907
title: "stdlib has nested if decision trees that should be match expressions"
area: stdlib
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-04-26
updated: 2026-04-26
target: stdlib/
---

# ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907: stdlib has nested if decision trees that should be match expressions

## 概要

有限のリテラル値・enum variant・token kind の分岐を深い if / else if 連鎖で書いている箇所があり、意図としては match で表現すべき判定が読みづらくなっている。今回の json_escape の文字 escape 分岐のように、compiler bug 回避や一時実装の名残として不自然な制御構造が stdlib に残り得る。

## 対象

- `stdlib/`

## 根拠

- `stdlib/alloc/encoding/json.nepl` の `json_escape` は、JSON 文字列 escape 対象の固定文字集合を `if` の深いネストで判定している。
- 同種の書き方は、compiler bug 回避や match lowering の制約を隠したまま stdlib に残る可能性があるため、stdlib 全体の監査対象として追跡する。

## 問題

有限のリテラル値・enum variant・token kind の分岐を深い if / else if 連鎖で書いている箇所があり、意図としては match で表現すべき判定が読みづらくなっている。今回の json_escape の文字 escape 分岐のように、compiler bug 回避や一時実装の名残として不自然な制御構造が stdlib に残り得る。

## 影響

stdlib の仕様分岐がデータごとの対応表として読めず、抜け漏れ・順序依存・将来の追加漏れをレビューで見つけにくい。compiler bug 回避の workaround が通常の実装として残ると、compiler 側の不具合や未整備な match lowering を隠してしまう。

## 修正方針

stdlib 全体を監査し、有限集合の分岐・literal dispatch・variant dispatch は原則 match へ置き換える。match 化できない場合は stdlib 側の workaround で済ませず、阻害している compiler bug を別 Issue として登録し、回帰テストを追加する。json_escape の文字 escape 分岐はこの方針の先行対象として扱う。

## 検証

stdlib の対象箇所を match 化した上で、該当 module の doctest と stdlib 全体 doctestを実行する。必要に応じて literal match / wildcard match / enum match の compiler regression test を追加し、不自然な if 連鎖が再発していないことを静的検索で確認する。
