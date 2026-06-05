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

`nodesrc/test_stdlib_documentation_contract.js` の current baseline を再集計した時点で、stdlib は `declarationNoDoc=361`、`declarationNoDoctest=1690`、`publicDeclarationNoDoctest=1531` を持つ。これは Zenn 記事の「契約、現状実装、enum の場合分け、計算量、simple/typical example、doc test」を doc comment に書く方針に対して未達である。

2026-06-05 の BitSet slice で `stdlib/alloc/collections/bitset` の facade / type / layout / storage / mutation / diagnostic helper docs と report doctest を追加し、baseline は `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` まで改善した。ただし adjacency_matrix / binary_heap / bloom_filter / btree などに declaration doc gap が残るため、この issue は open のまま継続する。

## 対象

- `stdlib/core`
- `stdlib/alloc`
- `stdlib/std`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` の再集計で、current baseline は `files=456`、`declarationNoDoc=361`、`declarationNoDoctest=1690` だった。
- `stdlib/alloc/collections/adjacency_matrix/layout.nepl` の layout helper 5件には doc comment と doctest を追加済みだが、sample gaps には `stdlib/alloc/collections/adjacency_matrix/api/*`、`stdlib/alloc/collections/adjacency_matrix/storage.nepl`、`stdlib/alloc/collections/binary_heap/*` などの declaration doc 欠落が残る。
- baseline refresh はこれ以上の悪化を止める regression guard であり、既存 gap を解消したことを意味しない。
- `stdlib/alloc/collections/bitset` では、owner-backed `BitSetUpdateError` を直接構築する doctest を避け、public `insert` / `remove` の Err 経路から error を取得して `bitset_update_error_diag` と `bitset_update_error_owner` の契約を確認する形にした。

## 問題

現状の stdlib は module doc の欠落は 0 だが、declaration 単位では doc comment と doctest が不足している。public API の contract と current implementation が宣言近傍にないため、型だけでは分からない所有権、計算量、error enum の条件、境界条件を利用者や reviewer が確認しにくい。

2026-06-05 時点で、baseline は現在値まで締め直した。これにより既存 gap の悪化は検査で止まるが、`declarationNoDoc=361` と `declarationNoDoctest=1690` はまだ未解決の負債であるため、この issue は open のままとする。宣言検出そのものが減って gap が隠れることを防ぐため、`declarations=2525` も下限として検査する。

同日 BitSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` である。`nodesrc/test_stdlib_bitset_doc_report_contract.js` により、BitSet の report doctest と owner recovery doc contract は total count だけでなく module 固有にも固定する。

## 影響

stdlib の修正時に、契約ではなく実装断片や既存挙動の記憶へ依存しやすくなる。特に collection / IO / GUI のように owner、Result、capability、platform boundary が絡む module では、doc gap が静的検査の活用不足やテスト観点漏れにつながる。

## 修正方針

module family ごとに分割して、declaration doc と declaration doctest を減らす。単純な baseline 下げではなく、各 public API について contract、現在の実装、計算量、Result / Option / enum の分岐条件、simple example と typical example を追加する。helper-only private declaration は、module doc または近傍の public doctestで検証される場合に限り、その根拠を記す。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`
- module family ごとの focused doctest
- 追加される cfg-test-style regular tests
- `node nodesrc/test_stdlib_bitset_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl -i stdlib/alloc/collections/bitset/types.nepl -i stdlib/alloc/collections/bitset/layout.nepl -i stdlib/alloc/collections/bitset/storage.nepl -i stdlib/alloc/collections/bitset/mutation.nepl -i stdlib/alloc/collections/bitset/api.nepl -i stdlib/alloc/collections/bitset/api/diagnostic.nepl -i stdlib/alloc/collections/bitset/api/create.nepl -i stdlib/alloc/collections/bitset/api/observer.nepl -i stdlib/alloc/collections/bitset/api/update.nepl -i stdlib/alloc/collections/bitset/api/bulk.nepl -i stdlib/alloc/collections/bitset/api/cleanup.nepl -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/agent2-bitset-doc-slice-2.json -j 1 --dist web/dist --assert-io`
