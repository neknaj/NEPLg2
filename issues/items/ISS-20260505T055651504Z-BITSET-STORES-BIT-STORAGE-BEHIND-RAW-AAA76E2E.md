---
id: ISS-20260505T055651504Z-BITSET-STORES-BIT-STORAGE-BEHIND-RAW-AAA76E2E
title: "BitSet stores bit storage behind raw MemPtr bytes"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "stdlib/alloc/collections/bitset.nepl, nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js"
---

# ISS-20260505T055651504Z-BITSET-STORES-BIT-STORAGE-BEHIND-RAW-AAA76E2E: BitSet stores bit storage behind raw MemPtr bytes

## 概要

BitSet keeps its backing bit array as MemPtr<u8> and updates it through raw byte load/store helpers. The owner state is therefore a raw memory range rather than a typed collection field that Resource IR and source policies can reason about.

## 対象

- `stdlib/alloc/collections/bitset.nepl, nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/bitset.nepl` は `BitSet.bits <MemPtr<u8>>` を owner field として持ち、`new` は `alloc_ptr<u8>`、`contains` / `insert` / `remove` / `clear` / `fill` は `load_u8` / `store_u8` と `mem_ptr_addr` で bit storage を更新していた。
- `free` は `MemPtr<u8>` を取り出して `dealloc_raw` へ渡すため、Resource IR から見ると BitSet の backing storage は typed field owner ではなく raw byte range だった。
- `nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js` も raw owner cleanup を要求しており、typed storage への移行を regression policy で守れていなかった。

## 問題

BitSet keeps its backing bit array as MemPtr<u8> and updates it through raw byte load/store helpers. The owner state is therefore a raw memory range rather than a typed collection field that Resource IR and source policies can reason about.

## 影響

Static memory-safety checks must keep treating a BitSet byte range as an opaque raw allocation. This preserves raw owner discipline in a public collection and makes regressions toward raw storage hard to distinguish from legitimate byte operations.

## 修正方針

Migrate BitSet backing storage to a typed Vec<i32> byte-value array, update bit operations to use Vec get/replace through borrowed storage, and strengthen the source policy so MemPtr/raw load/store/dealloc cannot return.

## 検証

Run the BitSet source policy and focused BitSet doctests, then regenerate and check the issue index.

## 対応結果

`BitSet` の backing storage を `MemPtr<u8>` から `Vec<i32>` の byte-value storage へ移行した。

- `BitSet.bits` は `Vec<i32>` owner field になり、public collection の owner state が raw pointer ではなく typed field として見えるようになった。
- `new` は `vec::filled<i32> nbytes 0` で全 byte slot を初期化する。
- `contains` は borrowed `Vec` に対する `vec::get` で byte value を読み、範囲外は従来どおり `Diag` として返す。
- `insert` / `remove` / `clear` / `fill` は borrowed `Vec` に対する `vec::replace` で更新し、mutation を伴う `clear` / `fill` は impure signature に揃えた。
- `free` は `Vec<i32>` owner を `field::get` で取り出し、`vec::free<i32>` で閉じる。
- `nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js` は `MemPtr` / raw alloc/load/store/dealloc が BitSet 実装へ戻らないことと、typed `Vec<i32>` storage contract を検査するように更新した。

検証:

- `node nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_bitset_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_bitset_update_error_owner.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl --no-tree -o tmp/bitset-module-typed-storage-agent1.json -j 1 --dist web/dist`: total=7, passed=7
- `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/bitset-typed-storage-agent1.json -j 1 --dist web/dist`: total=6, passed=6
