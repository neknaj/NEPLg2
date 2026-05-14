---
id: ISS-20260514T234418963Z-HASHMAP-AND-HASHSET-ROOT-FACADES-RE--68724B49
title: "HashMap and HashSet root facades re-export internal storage helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, nodesrc/test_stdlib_hashmap_storage_contract.js, nodesrc/test_stdlib_hashset_storage_contract.js, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260514T234418963Z-HASHMAP-AND-HASHSET-ROOT-FACADES-RE--68724B49: HashMap and HashSet root facades re-export internal storage helpers

## 概要

HashMap and HashSet root facades publicly merge storage/probe/rehash implementation modules, exposing storage allocation and probing helpers through ordinary safe imports.

## 対象

- `stdlib/alloc/collections/hashmap.nepl`
- `stdlib/alloc/collections/hashset.nepl`
- `nodesrc/test_stdlib_hashmap_storage_contract.js`
- `nodesrc/test_stdlib_hashset_storage_contract.js`
- `tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `alloc/collections/hashmap` と `alloc/collections/hashset` は ordinary safe facade だが、root が `storage` / `probe` / `rehash` を `pub #import ... as @merge` していた。
- `hashmap_alloc_storage` / `hashset_alloc_storage` は typed storage owner を直接返す implementation helper であり、public `new` / `with_capacity` / `insert` / `free` の invariant を経由しない。
- compiler 側では owner-backed aggregate constructor / field projection を構造判定で拒否するようにしたが、stdlib root facade が internal helper 名を通常 import 面に残すと、Stage 6 の public API boundary が review しにくくなる。

## 問題

HashMap and HashSet root facades publicly merge storage/probe/rehash implementation modules, exposing storage allocation and probing helpers through ordinary safe imports.

## 影響

Ordinary collection users can depend on internal storage helper names from the safe facade, widening the Stage 6 public/raw boundary and weakening Resource IR assumptions about owner-backed aggregate construction and projection.

## 修正方針

Keep root facades limited to public types and public API. Internal modules should continue importing storage/probe/rehash explicitly, while source policy and compile-fail doctests prove the helpers are not visible from root imports.

## 検証

Run focused HashMap/HashSet source policies, focused collection cleanup doctests, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 解決

`alloc/collections/hashmap` と `alloc/collections/hashset` の root facade から、`storage` / `probe` / `rehash` の public `@merge` re-export を削除した。root は public 型と public API だけを再公開し、storage allocation、probe、rehash helper は `api.nepl` / implementation module が明示 import して使う境界に閉じる。

同時に、HashMap root doctest が root facade 経由の偶発的な helper visibility に依存しないよう、`core/math` と `alloc/string` を明示 import し、`string::len` を使う形へ更新した。これにより `alloc/collections/hashmap` root は private implementation import を持たない facade として固定できる。

回帰として、source policy で root facade が `storage` / `probe` / `rehash` を public merge しないこと、private implementation import を持たないことを検査する。さらに `tests/stdlib/collection_cleanup_contract.n.md` に、root import だけでは `hashmap_alloc_storage` / `hashset_alloc_storage` が `resolve.identifier.undefined` になる compile-fail doctest を追加した。

検証:

- `node nodesrc/test_stdlib_hashmap_storage_contract.js`
- `node nodesrc/test_stdlib_hashset_storage_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-hash-root-facade-cleanup.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl --no-tree -o tmp/agent1-hash-root-facade-modules.json -j 1 --dist web/dist --assert-io`
