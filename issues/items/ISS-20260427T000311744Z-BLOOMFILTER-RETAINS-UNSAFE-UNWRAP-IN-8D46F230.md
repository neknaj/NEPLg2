---
id: ISS-20260427T000311744Z-BLOOMFILTER-RETAINS-UNSAFE-UNWRAP-IN-8D46F230
title: "BloomFilter retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/bloom_filter.nepl, tests/stdlib/bloom_filter_collections.n.md, nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js"
---

# ISS-20260427T000311744Z-BLOOMFILTER-RETAINS-UNSAFE-UNWRAP-IN-8D46F230: BloomFilter retains unsafe unwrap in owned bit storage cleanup

## 概要

BloomFilter.free still calls uwok on dealloc_ptr for its owned bit array while public allocation APIs expose Result.

## 対象

- `stdlib/alloc/collections/bloom_filter.nepl, tests/stdlib/bloom_filter_collections.n.md, nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js`

## 根拠

- `BloomFilter.new` は `nbits > 0` のときだけ `nbytes > 0` の bit array を確保し、成功時の `bits` pointer を `BloomFilter` owner に格納する。
- `BloomFilter.free` は generic struct の field を読むため一時領域へ保存した後、その owned `bits` を `dealloc_ptr<u8>` に渡し、`Result` を `uwok` していた。
- bit array cleanup の前提は owner invariant で保証されるため、checked deallocation の Err arm を unsafe helper で握りつぶす必要はない。

## 問題

BloomFilter.free still calls uwok on dealloc_ptr for its owned bit array while public allocation APIs expose Result.

## 影響

Probabilistic membership filters for self-host caches keep a trap-prone cleanup path and weaken the collection-wide unsafe-helper policy.

## 修正方針

Replace the owned bit-array cleanup with dealloc_raw, document the invariant, add a free smoke regression, and register a source guard.

## 解決内容

- `BloomFilter.free` の bit array cleanup を `dealloc_ptr + uwok` から `dealloc_raw mem_ptr_addr bits nbytes` に変更した。
- generic struct field 読み取り用の一時 `bf_mem` は従来どおり `dealloc_raw` で解放し、所有 bit array と一時 struct 領域の責務を分けた。
- `free` の doc comment に、`new` が確保した `nbytes > 0` の bit array を `BloomFilter` が所有していること、`free` 後の再利用は禁止であることを明記した。
- `tests/stdlib/bloom_filter_collections.n.md` に `bloom_filter_free_releases_owned_storage` を追加し、free 後に再確保できることと再確保した owner も free できることを確認した。
- `nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js` を追加し、BloomFilter 実装に unsafe unwrap helper / unreachable が戻らないことと、`free` が raw owner cleanup を使うことを CI source policy に登録した。

## 検証

- `node nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js`: pass
- source policy regressions: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/bloom_filter.nepl --no-tree -o tmp/bloom-filter-owned-cleanup-docs-after-b99686a.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib/bloom_filter_collections.n.md -i stdlib/tests/bloom_filter.n.md --no-tree -o tmp/bloom-filter-owned-cleanup-focused-after-b99686a.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-bloom-filter-owned-cleanup-after-b99686a.json -j 4`: 289/289 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-bloom-filter-owned-cleanup-after-b99686a.json -j 4`: 418/418 passed
