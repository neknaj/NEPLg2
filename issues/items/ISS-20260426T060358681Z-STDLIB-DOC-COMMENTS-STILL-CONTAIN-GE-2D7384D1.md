---
id: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
title: "stdlib doc comments still contain generated boilerplate"
area: stdlib
status: open
resolved: false
priority: P2
type: doc
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/alloc/collections/vec/sort.nepl, stdlib/core/cast.nepl"
---

# ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1: stdlib doc comments still contain generated boilerplate

## 概要

doc/stdlib_doc_comment_policy.md は手書きで具体的な説明を書く方針だが、stdlib の複数ファイルに 主な用途、定義済み処理をそのまま呼び出す薄いラッパ、引数の値は関数呼び出しで移動するため再利用時は束縛し直す といった汎用テンプレート文が大量に残っている。

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/encoding/json.nepl, stdlib/nm/parser.nepl, stdlib/alloc/collections/vec/sort.nepl, stdlib/core/cast.nepl`

## 根拠

- 未記入

## 問題

doc/stdlib_doc_comment_policy.md は手書きで具体的な説明を書く方針だが、stdlib の複数ファイルに 主な用途、定義済み処理をそのまま呼び出す薄いラッパ、引数の値は関数呼び出しで移動するため再利用時は束縛し直す といった汎用テンプレート文が大量に残っている。

## 影響

コメントが API 固有のアルゴリズム、所有権、制約、計算量を説明せず、利用者や self-host 実装者が実装を誤読しやすい。コメントの品質問題が実装レビューのノイズにもなる。

## 修正方針

ファイル単位で public API と内部 helper を分け、テンプレート文を実際の責務、失敗条件、所有権、計算量に置き換える。大きいファイルは RV-STDLIB-009 の分割と合わせて進める。

## 検証

対象ファイルで boilerplate marker を検索し、代表 API に具体的な doc comment と doctest があることを確認する。
