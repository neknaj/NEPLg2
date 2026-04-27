---
id: ISS-20260427T000311941Z-COUNTINGBLOOMFILTER-RETAINS-UNSAFE-U-D4E48E30
title: "CountingBloomFilter retains unsafe unwrap in owned counter cleanup"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/counting_bloom_filter.nepl, tests/stdlib/counting_bloom_filter_collections.n.md, nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js"
---

# ISS-20260427T000311941Z-COUNTINGBLOOMFILTER-RETAINS-UNSAFE-U-D4E48E30: CountingBloomFilter retains unsafe unwrap in owned counter cleanup

## 概要

CountingBloomFilter.free still calls uwok on dealloc_ptr for the owned counter array.

## 対象

- `stdlib/alloc/collections/counting_bloom_filter.nepl, tests/stdlib/counting_bloom_filter_collections.n.md, nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`

## 根拠

- `CountingBloomFilter.new` は `nslots > 0` のときだけ `nbytes > 0` の counter 配列を確保し、成功時の `counters` pointer を `CountingBloomFilter` owner に格納する。
- `CountingBloomFilter.free` は generic struct の field を読むため一時領域へ保存した後、その owned `counters` を `dealloc_ptr<u8>` に渡し、`Result` を `uwok` していた。
- counter 配列 cleanup の前提は owner invariant で保証されるため、checked deallocation の Err arm を unsafe helper で握りつぶす必要はない。

## 問題

CountingBloomFilter.free still calls uwok on dealloc_ptr for the owned counter array.

## 影響

Counting filter cleanup remains inconsistent with Result-returning constructors and can hide ownership invariant bugs behind unreachable traps.

## 修正方針

Use dealloc_raw for owned counter storage, document the invariant, add a cleanup regression, and prevent unsafe helpers from returning to the implementation.

## 解決内容

- `CountingBloomFilter.free` の counter cleanup を `dealloc_ptr + uwok` から `dealloc_raw mem_ptr_addr counters nbytes` に変更した。
- generic struct field 読み取り用の一時 `bf_mem` は従来どおり `dealloc_raw` で解放し、所有 counter 配列と一時 struct 領域の責務を分けた。
- `free` の doc comment に、`new` が確保した `nbytes > 0` の counter 配列を `CountingBloomFilter` が所有していること、`free` 後の再利用は禁止であることを明記した。
- `tests/stdlib/counting_bloom_filter_collections.n.md` に `counting_bloom_filter_free_releases_owned_storage` を追加し、free 後に再確保できることと再確保した owner も free できることを確認した。
- `nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js` を追加し、CountingBloomFilter 実装に unsafe unwrap helper / unreachable が戻らないことと、`free` が raw owner cleanup を使うことを CI source policy に登録した。

## 検証

- `node nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`: pass
- source policy regressions: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-tree -o tmp/counting-bloom-filter-owned-cleanup-docs-after-32f5c78.json -j 1`: 6/6 passed
- `node nodesrc/tests.js -i tests/stdlib/counting_bloom_filter_collections.n.md -i stdlib/tests/counting_bloom_filter.n.md --no-tree -o tmp/counting-bloom-filter-owned-cleanup-focused-after-32f5c78.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-counting-bloom-filter-owned-cleanup-after-32f5c78.json -j 4`: 290/290 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-counting-bloom-filter-owned-cleanup-after-32f5c78.json -j 4`: 418/418 passed
