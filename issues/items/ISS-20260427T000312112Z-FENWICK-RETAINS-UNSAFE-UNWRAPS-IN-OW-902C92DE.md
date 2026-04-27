---
id: ISS-20260427T000312112Z-FENWICK-RETAINS-UNSAFE-UNWRAPS-IN-OW-902C92DE
title: "Fenwick retains unsafe unwraps in owned tree internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/alloc/collections/fenwick.nepl, tests/stdlib/fenwick_collections.n.md, nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js"
---

# ISS-20260427T000312112Z-FENWICK-RETAINS-UNSAFE-UNWRAPS-IN-OW-902C92DE: Fenwick retains unsafe unwraps in owned tree internals

## 概要

Fenwick initialization, update, and free paths call uwok on checked store_i32/dealloc_ptr even though the tree owns the backing array.

## 対象

- `stdlib/alloc/collections/fenwick.nepl, tests/stdlib/fenwick_collections.n.md, nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js`

## 根拠

- `Fenwick.new` は `n + 1` 要素の 1-indexed `bit` 配列を確保し、`Fenwick` owner に格納する。
- 初期化と `add` は owner 配列内の index だけを操作するにもかかわらず、checked `store_i32` の `Result` を `uwok` していた。
- `Fenwick.free` は owned `bit` 配列を `dealloc_ptr<i32>` に渡し、`Result` を `uwok` していた。
- これらは owner invariant で保証される内部操作なので、通常処理を unsafe helper に依存させる必要はない。

## 問題

Fenwick initialization, update, and free paths call uwok on checked store_i32/dealloc_ptr even though the tree owns the backing array.

## 影響

Self-host frequency/prefix-sum helpers can trap on internal bookkeeping paths, and allocation cleanup semantics remain inconsistent across collections.

## 修正方針

Introduce raw owned-array store helpers, replace owned cleanup with dealloc_raw, keep public failures as Result, and add source and behavior regressions.

## 解決内容

- `fenwick_ptr_at` / `fenwick_load_owned` / `fenwick_store_owned` を追加し、所有済み `bit` 配列への raw access を内部 helper に集約した。
- `new` の初期化と `add` の更新から `uwok store_i32` を削除し、owner 配列への raw store に変更した。
- `fenwick_sum_prefix_raw` も同じ owned load helper を使うようにし、Fenwick 内部配列 access の前提を一箇所に揃えた。
- `free` を `dealloc_ptr + uwok` から `dealloc_raw mem_ptr_addr bit bytes` に変更し、doc comment に owner invariant と free 後の再利用禁止を明記した。
- `tests/stdlib/fenwick_collections.n.md` に `fenwick_free_releases_owned_storage` を追加し、free 後に再確保できることと再確保した owner も free できることを確認した。
- `nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js` を追加し、Fenwick 実装に unsafe unwrap helper / unreachable が戻らないことと、raw owner cleanup/access helper が維持されることを CI source policy に登録した。

## 検証

- `node nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js`: pass
- source policy regressions: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl --no-tree -o tmp/fenwick-owned-cleanup-docs.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i tests/stdlib/fenwick_collections.n.md -i stdlib/tests/fenwick.n.md --no-tree -o tmp/fenwick-owned-cleanup-focused.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-fenwick-owned-cleanup.json -j 4`: 291/291 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-fenwick-owned-cleanup.json -j 4`: 418/418 passed

## 2026-04-28 再発確認と追加解決

### 再現

- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl --no-tree -o tmp/fenwick-raw-temp-regression-before.json -j 1`: 5 件中 4 件が `D3100 deallocating raw memory place containing non-Copy value: fw_mem` で失敗した。
- 失敗箇所は `add` / `sum_prefix` / `sum_range` / `free` の doctest で、いずれも field 観察用に `Fenwick` owner を `fw_mem` raw temporary へ格納した後、owner を consume しないまま `dealloc_raw fw_mem size_of<Fenwick>` していた。

### 根本原因

`get load<Fenwick> fw_mem "field"` は field を観察するだけで raw temporary 内の `Fenwick` owner を move しない。したがって、`fw_mem` を raw dealloc すると `bit` owner を含む `Fenwick` 値を破棄する形になり、強化された raw memory checker が live owner discard として拒否する。

### 追加修正

- `add` / `sum_prefix` / `sum_range` / `free` から `fw_mem` の `alloc_raw` / `store` / `load` / `dealloc_raw` を削除した。
- `n` と `bit` は `field::get fw "n"` / `field::get fw "bit"` で直接読むようにした。
- `add` の成功時は元の `fw` owner を `Ok` として返し、失敗時は従来どおり `Diag` を返す。
- owned `bit` 配列の解放責務は `free` の `dealloc_raw mem_ptr_addr bit bytes` に限定し、struct 観察用 temporary と backing storage の責務を分離した。

### 追加検証

- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl --no-tree -o tmp/fenwick-raw-temp-regression-after.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i tests/stdlib/fenwick_collections.n.md -i stdlib/tests/fenwick.n.md --no-tree -o tmp/fenwick-raw-temp-focused-after.json -j 1`: 4/4 passed
- `node nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js`: pass
