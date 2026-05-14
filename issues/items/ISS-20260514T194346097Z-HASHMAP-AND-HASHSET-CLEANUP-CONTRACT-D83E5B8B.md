---
id: ISS-20260514T194346097Z-HASHMAP-AND-HASHSET-CLEANUP-CONTRACT-D83E5B8B
title: "HashMap and HashSet cleanup contract misses key and hasher Copy-bound regressions"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_hashmap_storage_contract.js, nodesrc/test_stdlib_hashset_storage_contract.js"
---

# ISS-20260514T194346097Z-HASHMAP-AND-HASHSET-CLEANUP-CONTRACT-D83E5B8B: HashMap and HashSet cleanup contract misses key and hasher Copy-bound regressions

## 概要

HashMap / HashSet の `free` は key / value / hasher を Copy-only cleanup 境界に閉じているが、回帰テストは HashMap value しか直接確認しておらず、source policy も `free` signature を固定していなかった。

## 対象

- `tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_hashmap_storage_contract.js, nodesrc/test_stdlib_hashset_storage_contract.js`

## 根拠

- `HashMap.free` は `.K: HashKey&Copy,.V: Copy,.H: Hasher<.K>&Copy` を要求する。
- `HashSet.free` は `.T: HashKey&Copy,.H: Hasher<.T>&Copy` を要求する。
- `tests/stdlib/collection_cleanup_contract.n.md` は HashMap value の non-Copy 拒否を確認していたが、HashMap key / HashMap hasher / HashSet key / HashSet hasher の cleanup 境界を個別に確認していなかった。
- `nodesrc/test_stdlib_hashmap_storage_contract.js` と `nodesrc/test_stdlib_hashset_storage_contract.js` は typed storage owner release は見ていたが、`free` の Copy-only signature そのものは固定していなかった。

## 問題

HashMap / HashSet の `free` signature が将来緩んでも、既存の doctest / source policy では key や hasher 側の non-Copy cleanup 退行を検出できない可能性があった。

## 影響

`RV-STDLIB-004` は `OwnedBuffer<T>` / initialized prefix / element Drop traversal が完成するまで、Copy-only cleanup 境界で unsupported non-Copy payload を拒否する方針に依存している。key / hasher 側の coverage がないと、field-level Drop traversal なしに owner-bearing payload を破棄できるように見える退行を隠す。

## 修正方針

HashSet key、HashSet hasher、HashMap key、HashMap hasher それぞれの non-Copy cleanup を独立した compile_fail doctest にする。さらに HashMap / HashSet の storage contract source policy で `free` signature の Copy-only 境界を直接固定する。

## 検証

Run the focused collection cleanup doctests, HashMap/HashSet storage policy checks, issue index validation, and diff whitespace checks.

## 修正結果

- `tests/stdlib/collection_cleanup_contract.n.md` に HashSet key / HashSet hasher / HashMap key / HashMap hasher の独立 compile-fail regression を追加した。
- custom key は `HashKey` を実装するが `Copy` を実装しない型にし、`HashKey` が Copy 証明にならないことを cleanup 境界で固定した。
- custom hasher は `Hasher<i32>` を実装するが `Copy` を実装しない型にし、`Hasher` が hasher field の破棄許可にならないことを cleanup 境界で固定した。
- HashMap / HashSet の storage contract source policy に、`free` が Copy-only key/value/hasher contract を公開することを追加した。

## 検証結果

- `node nodesrc/test_stdlib_hashmap_storage_contract.js`: pass
- `node nodesrc/test_stdlib_hashset_storage_contract.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-hash-collection-cleanup-contract.json -j 1 --dist web/dist --assert-io`: 20/20 pass

## 関連

- Parent: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
