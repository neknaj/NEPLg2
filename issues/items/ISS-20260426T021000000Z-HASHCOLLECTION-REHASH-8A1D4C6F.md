---
id: ISS-20260426T021000000Z-HASHCOLLECTION-REHASH-8A1D4C6F
title: "HashMap and HashSet have fixed capacity and no rehash path"
area: stdlib
status: verified
resolved: true
priority: P1
type: performance
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl"
source: doc/neplg2/pre_selfhost_performance_audit_20260426.md
---

# ISS-20260426T021000000Z-HASHCOLLECTION-REHASH-8A1D4C6F: HashMap and HashSet have fixed capacity and no rehash path

## 概要

`HashMap` / `HashSet` は open addressing + linear probing を使うが、容量は `new` 時の 16 に固定され、自動 rehash / grow がない。
満杯になると `Diag::CapacityExceeded` で失敗し、load factor が高い状態では probe が長くなる。

## 根拠

- `stdlib/alloc/collections/hashmap.nepl:9` は固定長 hash table と明記している。
- `hashmap.nepl:124` の `new` は `cap = 16` 固定。
- `hashmap.nepl:275` は満杯時に `diag_capacity_exceeded "hashmap_insert"` を返す。
- `stdlib/alloc/collections/hashset.nepl` も同じ構造で、`new` が `cap = 16`、満杯時に `hashset_insert` で capacity exceeded になる。

## 問題

self-host compiler の symbol table、intern table、module table、type table は 16 要素を容易に超える。
現状のままだと、性能低下以前に容量上限でコンパイル不能になる。
また、削除後の tombstone が増えると実要素数が少なくても probe が長くなる。

## 影響

lexer / parser の小さな table 以外に `HashMap` / `HashSet` を使いにくくなり、self-host 実装が `Vec` 線形探索や ad hoc table へ逃げる原因になる。
これは計算量を悪化させ、後で collection を差し替える修正範囲を広げる。

## 修正方針

load factor 閾値を決め、`insert` 前に grow + rehash を行う。
初期容量指定 API も追加し、self-host compiler の用途では table size 見積もりから `with_capacity` を使えるようにする。
tombstone が多い場合は同容量 rehash で probe chain を短縮する。

## 対応

`HashMap` / `HashSet` の header に tombstone 数を追加し、`insert` 前に既存 key を確認した上で `count + tombstones + 1` が 75% load limit を超える場合に rehash するようにした。
実要素数だけなら余裕がある場合は同容量 rehash、実要素数も閾値を超える場合は容量を 2 倍に grow する。
`new` は内部 constructor を通じて初期容量 16 を使い、公開 API として `with_capacity` を追加した。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/hash_collection_rehash.n.md --no-tree -o tmp/hash-collection-rehash.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl --no-tree -o tmp/hash-collection-stdlib-doctests-after-comments.json -j 1`: 14/14 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/stdlib/collections_diag.n.md -i tests/stdlib/traits_hash.n.md -i tests/stdlib/pipe_collections.n.md -i tests/stdlib/hash_collection_rehash.n.md --no-tree -o tmp/hash-collection-rehash-suite-final.json -j 1`: 28/28 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-hash-collection-rehash.json`: 13/13 passed
