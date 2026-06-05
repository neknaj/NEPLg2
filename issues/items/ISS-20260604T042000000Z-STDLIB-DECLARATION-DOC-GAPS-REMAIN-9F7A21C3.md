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

2026-06-05 の BitSet slice で `stdlib/alloc/collections/bitset` の facade / type / layout / storage / mutation / diagnostic helper docs と report doctest を追加し、baseline は `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` まで改善した。

同日の AdjacencyMatrix slice で `stdlib/alloc/collections/adjacency_matrix` の facade / type / storage / mutation / diagnostic / observer / update / bulk / cleanup docs と report doctest を追加し、baseline は `moduleNoDoctest=301`、`declarationNoDoc=343`、`declarationNoDoctest=1679`、`publicDeclarationNoDoctest=1520` まで改善した。ただし binary_heap / bloom_filter / btree などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BinaryHeap slice で `stdlib/alloc/collections/binary_heap` の facade / type invariant / pop result / observer / pop API / storage helper / order helper docs と report doctest を追加し、baseline は `moduleNoDoctest=299`、`declarationNoDoc=332`、`declarationNoDoctest=1671`、`publicDeclarationNoDoctest=1512` まで改善した。ただし bloom_filter / btree などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BloomFilter slice で `stdlib/alloc/collections/bloom_filter` の facade / type invariant / hash helper / layout helper / storage helper / mutation helper / public API docs と report doctest を追加し、baseline は `moduleNoDoctest=297`、`declarationNoDoc=318`、`declarationNoDoctest=1670`、`publicDeclarationNoDoctest=1511` まで改善した。ただし btreemap / btreeset / counting_bloom_filter などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の CountingBloomFilter slice で `stdlib/alloc/collections/counting_bloom_filter` の facade / type invariant / hash helper / storage helper / mutation helper / public API docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=306`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし btreemap / btreeset / disjoint_set などに declaration doc gap が残るため、この issue は open のまま継続する。

同日の BTreeMap slice で `stdlib/alloc/collections/btreemap/search.nepl` と `storage.nepl` の search / owner-backed storage helper docs と report doctest を追加し、baseline は `moduleNoDoctest=295`、`declarationNoDoc=287`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` まで改善した。ただし btreeset / disjoint_set / fenwick などに declaration doc gap が残るため、この issue は open のまま継続する。

## 対象

- `stdlib/core`
- `stdlib/alloc`
- `stdlib/std`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` の再集計で、current baseline は `files=456`、`declarationNoDoc=361`、`declarationNoDoctest=1690` だった。
- `stdlib/alloc/collections/adjacency_matrix/layout.nepl` の layout helper 5件には doc comment と doctest を追加済みだったが、その後の BitSet / AdjacencyMatrix / BinaryHeap / BloomFilter / CountingBloomFilter / BTreeMap slice により sample gaps は btreeset / disjoint_set / fenwick 系へ進んでいる。
- baseline refresh はこれ以上の悪化を止める regression guard であり、既存 gap を解消したことを意味しない。
- `stdlib/alloc/collections/bitset` では、owner-backed `BitSetUpdateError` を直接構築する doctest を避け、public `insert` / `remove` の Err 経路から error を取得して `bitset_update_error_diag` と `bitset_update_error_owner` の契約を確認する形にした。

## 問題

現状の stdlib は module doc の欠落は 0 だが、declaration 単位では doc comment と doctest が不足している。public API の contract と current implementation が宣言近傍にないため、型だけでは分からない所有権、計算量、error enum の条件、境界条件を利用者や reviewer が確認しにくい。

2026-06-05 時点で、baseline は現在値まで締め直した。これにより既存 gap の悪化は検査で止まるが、`declarationNoDoc=361` と `declarationNoDoctest=1690` はまだ未解決の負債であるため、この issue は open のままとする。宣言検出そのものが減って gap が隠れることを防ぐため、`declarations=2525` も下限として検査する。

同日 BitSet slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=303`、`declarationNoDoc=350`、`declarationNoDoctest=1686`、`publicDeclarationNoDoctest=1527` である。`nodesrc/test_stdlib_bitset_doc_report_contract.js` により、BitSet の report doctest と owner recovery doc contract は total count だけでなく module 固有にも固定する。

同日 AdjacencyMatrix slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=301`、`declarationNoDoc=343`、`declarationNoDoctest=1679`、`publicDeclarationNoDoctest=1520` である。`nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js` により、AdjacencyMatrix の facade lifecycle、type invariant、typed byte storage、mutation、diagnostic kind、borrowed observer、owner recovery doc contract を module 固有にも固定する。

同日 BinaryHeap slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=299`、`declarationNoDoc=332`、`declarationNoDoctest=1671`、`publicDeclarationNoDoctest=1512` である。`nodesrc/test_stdlib_binary_heap_doc_report_contract.js` により、BinaryHeap の facade lifecycle、type invariant、observer / pop API、`Vec Option .T` storage、index math、swap、sift-up / sift-down、`BinaryHeapPop` owner accessor doc contract を module 固有にも固定する。

同日 BloomFilter slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=297`、`declarationNoDoc=318`、`declarationNoDoctest=1670`、`publicDeclarationNoDoctest=1511` である。`nodesrc/test_stdlib_bloom_filter_doc_report_contract.js` により、BloomFilter の facade lifecycle、type invariant、invalid length error kind、borrowed observer、false positive / false negative contract、hash / layout / storage / mutation helper doc contract を module 固有にも固定する。

同日 CountingBloomFilter slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=306`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_counting_bloom_filter_doc_report_contract.js` により、CountingBloomFilter の facade lifecycle、type invariant、invalid length error kind、borrowed observer、false positive / false negative、counter saturation / lower-bound remove、typed counter storage、hash / storage / mutation helper doc contract を module 固有にも固定する。

同日 BTreeMap slice 後に baseline を再度締め直した。新しい悪化防止ラインは `moduleNoDoctest=295`、`declarationNoDoc=287`、`declarationNoDoctest=1668`、`publicDeclarationNoDoctest=1509` である。`nodesrc/test_stdlib_btree_search_doc_report_contract.js` と `nodesrc/test_stdlib_btreemap_storage_doc_report_contract.js` により、BTreeMap の lower_bound / is_at、`Vec Option .K` / `Vec Option .V` storage、partial allocation cleanup、owner recovery、grow failure、storage invariant failure、Copy boundary、O(cap) / O(len0) contract を module 固有にも固定する。

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
- `node nodesrc/test_stdlib_adjacency_matrix_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/alloc/collections/adjacency_matrix/types.nepl -i stdlib/alloc/collections/adjacency_matrix/layout.nepl -i stdlib/alloc/collections/adjacency_matrix/storage.nepl -i stdlib/alloc/collections/adjacency_matrix/mutation.nepl -i stdlib/alloc/collections/adjacency_matrix/api.nepl -i stdlib/alloc/collections/adjacency_matrix/api/diagnostic.nepl -i stdlib/alloc/collections/adjacency_matrix/api/create.nepl -i stdlib/alloc/collections/adjacency_matrix/api/observer.nepl -i stdlib/alloc/collections/adjacency_matrix/api/update.nepl -i stdlib/alloc/collections/adjacency_matrix/api/bulk.nepl -i stdlib/alloc/collections/adjacency_matrix/api/cleanup.nepl -i tests/stdlib/adjacency_matrix_collections.n.md -i stdlib/tests/adjacency_matrix.n.md --no-tree -o tmp/agent2-adjacency-matrix-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_binary_heap_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl -i stdlib/alloc/collections/binary_heap/types.nepl -i stdlib/alloc/collections/binary_heap/storage.nepl -i stdlib/alloc/collections/binary_heap/order.nepl -i stdlib/alloc/collections/binary_heap/api.nepl -i stdlib/alloc/collections/binary_heap/api/create.nepl -i stdlib/alloc/collections/binary_heap/api/observer.nepl -i stdlib/alloc/collections/binary_heap/api/push.nepl -i stdlib/alloc/collections/binary_heap/api/pop.nepl -i stdlib/alloc/collections/binary_heap/api/cleanup.nepl -i tests/stdlib/binary_heap_collections.n.md -i stdlib/tests/binary_heap.n.md --no-tree -o tmp/agent2-binary-heap-doc-slice.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_bloom_filter_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/bloom_filter.nepl -i stdlib/alloc/collections/bloom_filter/types.nepl -i stdlib/alloc/collections/bloom_filter/hash.nepl -i stdlib/alloc/collections/bloom_filter/layout.nepl -i stdlib/alloc/collections/bloom_filter/storage.nepl -i stdlib/alloc/collections/bloom_filter/mutation.nepl -i stdlib/alloc/collections/bloom_filter/api.nepl -i stdlib/tests/bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md --no-tree -o tmp/agent2-bloom-filter-doc-slice-fourth.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_counting_bloom_filter_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/counting_bloom_filter.nepl -i stdlib/alloc/collections/counting_bloom_filter/types.nepl -i stdlib/alloc/collections/counting_bloom_filter/hash.nepl -i stdlib/alloc/collections/counting_bloom_filter/storage.nepl -i stdlib/alloc/collections/counting_bloom_filter/mutation.nepl -i stdlib/alloc/collections/counting_bloom_filter/api.nepl --no-tree -o tmp/agent2-counting-bloom-filter-doc-modules.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/agent2-counting-bloom-filter-existing-tests.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_btree_search_doc_report_contract.js`
- `node nodesrc/test_stdlib_btreemap_storage_doc_report_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap/search.nepl -i stdlib/alloc/collections/btreemap/storage.nepl -i stdlib/tests/btreemap.n.md --no-tree -o tmp/agent2-btreemap-doc-slice.json -j 1 --dist web/dist --assert-io`
