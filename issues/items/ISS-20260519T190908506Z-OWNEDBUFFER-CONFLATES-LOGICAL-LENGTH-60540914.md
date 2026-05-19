---
id: ISS-20260519T190908506Z-OWNEDBUFFER-CONFLATES-LOGICAL-LENGTH-60540914
title: "OwnedBuffer conflates logical length and initialized prefix"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260519T190908506Z-OWNEDBUFFER-CONFLATES-LOGICAL-LENGTH-60540914: OwnedBuffer conflates logical length and initialized prefix

## 概要

OwnedBuffer<T> stores only len/cap/storage, so logical live length and initialized/drop-relevant prefix remain the same field. Stage 6 needs these states separated before non-Copy move-out and drop traversal can be soundly implemented.

## 対象

- `stdlib/alloc/collections/vec/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 は、`MemPtr = non-owning pointer`、free obligation owner、initialized/moved/drop state の分離を要求している。
- 修正前の `OwnedBuffer<T>` は `len/cap/storage` だけを持ち、logical live length と initialized/drop-relevant prefix を区別できなかった。

## 問題

OwnedBuffer<T> stores only len/cap/storage, so logical live length and initialized/drop-relevant prefix remain the same field. Stage 6 needs these states separated before non-Copy move-out and drop traversal can be soundly implemented.

## 影響

Future non-Copy Vec APIs would have no structural place to record moved/deinitialized slots separately from visible length, keeping collection cleanup dependent on comments and Copy-only source policy instead of typed state.

## 修正方針

Add an explicit initialized_len field to OwnedBuffer<T>, update all constructors and owner-preserving update paths to maintain len/initialized_len together for the current Copy-only Vec contract, and make source policy reject regressions to the old three-field buffer shape.

## 対応内容

- `OwnedBuffer<T>` を `len / initialized_len / cap / storage` に拡張し、`Vec<T>` facade 直下には storage metadata を戻さない構造を維持した。
- `vec_empty` / `vec_alloc_empty` / `filled` / `push` / `pop` / `drop_last` / `clear` / `map` / `filter` / `partition` / `take_while` / `drop_while` の buffer construction を更新した。
- 現行 `.T: Copy` contract では成功 path の `initialized_len` を `len` と同値に保ち、`push` / `pop` の owner-returning path では `initialized_len` を別 field として読み戻す。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、`initialized_len` field と旧 3-field constructor 退行の source policy を追加した。

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-ownedbuffer-init-prefix-vec.json -j 1 --dist web/dist --assert-io`: 52/52 passed
- `cargo test -p nepl-core owner_aggregate_boundary_accepts_nested -- --nocapture`: 2 passed
- `cargo fmt -p nepl-core -- --check`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
