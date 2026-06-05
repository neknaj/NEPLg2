---
id: ISS-20260604T042000000Z-STDLIB-DECLARATION-DOC-GAPS-REMAIN-9F7A21C3
title: "stdlib declaration documentation gaps remain high after baseline refresh"
area: stdlib
status: open
resolved: false
priority: P1
type: doc
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/core, stdlib/alloc, stdlib/std"
---

# ISS-20260604T042000000Z-STDLIB-DECLARATION-DOC-GAPS-REMAIN-9F7A21C3: stdlib declaration documentation gaps remain high after baseline refresh

## 概要

`nodesrc/test_stdlib_documentation_contract.js` の current baseline を再集計した時点で、stdlib は `declarationNoDoc=366`、`declarationNoDoctest=1690`、`publicDeclarationNoDoctest=1531` を持つ。これは Zenn 記事の「契約、現状実装、enum の場合分け、計算量、simple/typical example、doc test」を doc comment に書く方針に対して未達である。

## 対象

- `stdlib/core`
- `stdlib/alloc`
- `stdlib/std`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` の再集計で、current baseline は `files=456`、`declarationNoDoc=366`、`declarationNoDoctest=1690` だった。
- sample gaps には `stdlib/alloc/collections/adjacency_matrix/*`、`stdlib/alloc/collections/binary_heap/*` などの declaration doc 欠落が含まれる。
- baseline refresh はこれ以上の悪化を止める regression guard であり、既存 gap を解消したことを意味しない。

## 問題

現状の stdlib は module doc の欠落は 0 だが、declaration 単位では doc comment と doctest が不足している。public API の contract と current implementation が宣言近傍にないため、型だけでは分からない所有権、計算量、error enum の条件、境界条件を利用者や reviewer が確認しにくい。

2026-06-05 時点で、baseline は現在値まで締め直した。これにより既存 gap の悪化は検査で止まるが、`declarationNoDoc=366` と `declarationNoDoctest=1690` はまだ未解決の負債であるため、この issue は open のままとする。宣言検出そのものが減って gap が隠れることを防ぐため、`declarations=2525` も下限として検査する。

## 影響

stdlib の修正時に、契約ではなく実装断片や既存挙動の記憶へ依存しやすくなる。特に collection / IO / GUI のように owner、Result、capability、platform boundary が絡む module では、doc gap が静的検査の活用不足やテスト観点漏れにつながる。

## 修正方針

module family ごとに分割して、declaration doc と declaration doctest を減らす。単純な baseline 下げではなく、各 public API について contract、現在の実装、計算量、Result / Option / enum の分岐条件、simple example と typical example を追加する。helper-only private declaration は、module doc または近傍の public doctestで検証される場合に限り、その根拠を記す。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`
- module family ごとの focused doctest
- 追加される cfg-test-style regular tests
