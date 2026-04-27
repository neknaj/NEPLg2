---
id: ISS-20260427T000311423Z-BITSET-RETAINS-UNSAFE-UNWRAP-IN-OWNE-60B0934D
title: "BitSet retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/bitset.nepl, tests/stdlib/bitset_collections.n.md, nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js"
---

# ISS-20260427T000311423Z-BITSET-RETAINS-UNSAFE-UNWRAP-IN-OWNE-60B0934D: BitSet retains unsafe unwrap in owned bit storage cleanup

## 概要

BitSet.free still calls uwok on dealloc_ptr for storage owned by the BitSet value, so the normal cleanup path depends on an unsafe Result helper instead of the owner invariant.

## 対象

- `stdlib/alloc/collections/bitset.nepl, tests/stdlib/bitset_collections.n.md, nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`

## 根拠

- `BitSet.new` は `nbits > 0` のときだけ `nbytes > 0` の byte 配列を確保し、成功時の `bits` pointer を `BitSet` owner に格納する。
- `BitSet.free` はその owned `bits` を `dealloc_ptr<u8>` に渡し、`Result` を `uwok` していた。
- 通常 cleanup の前提は owner invariant で保証されるため、checked deallocation の Err arm を unsafe helper で握りつぶす必要はない。

## 問題

BitSet.free still calls uwok on dealloc_ptr for storage owned by the BitSet value, so the normal cleanup path depends on an unsafe Result helper instead of the owner invariant.

## 影響

Self-host set membership helpers can turn cleanup invariant regressions into unreachable traps, and RV-STDLIB-010 cannot be closed while collection internals keep unsafe helpers.

## 修正方針

Replace owned bit storage cleanup with dealloc_raw, document the owner invariant, add a focused free regression, and add a source guard that prevents unsafe unwrap helpers from returning to BitSet implementation.

## 解決内容

- `BitSet.free` を `dealloc_ptr + uwok` から `dealloc_raw mem_ptr_addr bits nbytes` に変更した。
- `free` の doc comment に、`new` が確保した `nbytes > 0` の byte 配列を `BitSet` が所有していること、`free` 後の再利用は禁止であることを明記した。
- `tests/stdlib/bitset_collections.n.md` に `bitset_free_releases_owned_storage` を追加し、free 後に再確保できることと再確保した owner も free できることを確認した。
- `nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js` を追加し、BitSet 実装に unsafe unwrap helper / unreachable が戻らないことと、`free` が raw owner cleanup を使うことを CI source policy に登録した。

## 検証

- `trunk build`: pass
- `node nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`: pass
- source policy regressions: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl --no-tree -o tmp/bitset-owned-cleanup-docs-after-70c1e27.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/stdlib/bitset_collections.n.md -i stdlib/tests/bitset.n.md --no-tree -o tmp/bitset-owned-cleanup-focused-after-70c1e27.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-bitset-owned-cleanup-after-70c1e27.json -j 4`: 287/287 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-bitset-owned-cleanup-after-70c1e27.json -j 4`: 418/418 passed
