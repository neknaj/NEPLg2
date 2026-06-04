---
id: ISS-20260604T034124823Z-HASHMAP-AND-HASHSET-PROBE-PATHS-USE--2A6F7DD4
title: "HashMap and HashSet probe paths use -1 sentinel instead of typed absence"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/alloc/collections/hashmap/probe.nepl, stdlib/alloc/collections/hashmap/api.nepl, stdlib/alloc/collections/hashmap/rehash.nepl, stdlib/alloc/collections/hashset/probe.nepl, stdlib/alloc/collections/hashset/api.nepl, stdlib/alloc/collections/hashset/rehash.nepl"
---

# ISS-20260604T034124823Z-HASHMAP-AND-HASHSET-PROBE-PATHS-USE--2A6F7DD4: HashMap and HashSet probe paths use -1 sentinel instead of typed absence

## 概要

Subagent audit found HashMap/HashSet probing returning and checking -1 as an absence/sentinel value. Zenn guidance prefers Option/Result/enum plus match, because sentinel integers cannot distinguish not-found, tombstone, full table, and corrupted storage states statically.

## 対象

- `stdlib/alloc/collections/hashmap/probe.nepl, stdlib/alloc/collections/hashmap/api.nepl, stdlib/alloc/collections/hashset/api.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found HashMap/HashSet probing returning and checking -1 as an absence/sentinel value. Zenn guidance prefers Option/Result/enum plus match, because sentinel integers cannot distinguish not-found, tombstone, full table, and corrupted storage states statically.

## 影響

Collection invariants are harder to verify, collision/tombstone logic can collapse distinct states, and future non-Copy/drop-capable map storage cannot reliably recover owner state on failure.

## 修正方針

Introduce Option i32 or a ProbeResult enum such as Found/Empty/Tombstone/Full, and return Result where storage invariants are violated. Migrate API callers to match on the typed result.

## 検証

Add collision, tombstone remove, reinsertion, full-table, and not-found regular tests, plus source policy rejecting public -1 probe sentinel checks.

## 修正結果

- `hashmap_find_present` / `hashset_find_present` は `Option i32` を返し、未発見を `None` として表すようにした。
- `hashmap_find_insert_slot` / `hashset_find_insert_slot` は `Option HashMapInsertSlot` / `Option HashSetInsertSlot` を返し、capacity 周回で insert slot が見つからない場合に仮 index を返さないようにした。
- `api.nepl` と `rehash.nepl` の caller は `lt idx 0` ではなく `match Option::Some` / `Option::None` で分岐するようにした。
- rehash 中に新 storage への移送 slot が得られない場合は、新 storage を解放し、旧 owner を `HashMapUpdateError` / `HashSetUpdateError` として返すようにした。
- source policy に `find_present` / `find_insert_slot` の `Option` 戻り型と、`-1` / `lt ... 0` sentinel 禁止を追加した。
- timeout しやすかった hash collection の monolithic doctest は、小さい regular doctest に分割した。

## 実行した検証

- `node nodesrc/test_stdlib_hashmap_storage_contract.js`
- `node nodesrc/test_stdlib_hashset_storage_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap -i stdlib/alloc/collections/hashset -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md -i tests/stdlib/hash_collection_rehash.n.md -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/agent2-hash-probe-option-focused-after-split.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`
- `git diff --check`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-hash-probe-option-playground-editor.json`
