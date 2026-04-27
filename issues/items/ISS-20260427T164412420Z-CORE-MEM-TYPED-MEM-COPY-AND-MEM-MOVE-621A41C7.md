---
id: ISS-20260427T164412420Z-CORE-MEM-TYPED-MEM-COPY-AND-MEM-MOVE-621A41C7
title: "core mem typed mem_copy and mem_move can duplicate non-Copy owners"
area: stdlib
status: open
resolved: false
priority: P1
type: security
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/core/mem.nepl, nepl-core/src/passes/move_check.rs, tests/stdlib/mem_bulk_copy.n.md, tests/compiler/move_effect.n.md"
---

# ISS-20260427T164412420Z-CORE-MEM-TYPED-MEM-COPY-AND-MEM-MOVE-621A41C7: core mem typed mem_copy and mem_move can duplicate non-Copy owners

## 概要

`stdlib/core/mem.nepl` の typed `mem_copy<T>` / `mem_move<T>` は `MemPtr<T>` を受け取るため安全寄りの API に見えるが、`T: Copy` 制約も source 側の move/invalidate もなく任意 `T` を byte copy できる。`T` が owning non-Copy value の場合、所有者を複製できる。

## 対象

- `stdlib/core/mem.nepl, nepl-core/src/passes/move_check.rs, tests/stdlib/mem_bulk_copy.n.md, tests/compiler/move_effect.n.md`

## 根拠

- `stdlib/core/mem.nepl:968` の `mem_copy<T>(MemPtr<T>, MemPtr<T>, i32)` は `T: Copy` 制約を持たず、`mem_ptr_addr` で raw address へ降りて byte count `count * size_of<T>` を raw `mem_copy` に渡す。
- `stdlib/core/mem.nepl:1003` の `mem_move<T>(MemPtr<T>, MemPtr<T>, i32)` も `T: Copy` 制約を持たず、source region を compiler ownership state 上で moved/uninitialized にしない。
- `stdlib/core/mem.nepl:745` / `796` の raw `mem_copy` / `mem_move` は byte列操作であり、`T` の ownership、drop obligation、initialized state を扱わない。
- `stdlib/core/traits/copy.nepl:151` / `155` により `MemPtr<T>` 自体は Copy で、ポインタ handle の複製が許される。
- `nepl-core/src/passes/move_check.rs:98` 以降の raw memory place state は `load<T>` / `store<T>` の ownership を中心に追跡しているが、typed bulk copy/move で source/destination の initialized state を移す概念を持たない。

## 問題

`mem_copy<T>` は non-Copy owner を明示 clone なしで複製できる。`mem_move<T>` も名前は move だが、source の raw place を moved にせず、destination の既存 initialized value の drop obligation も扱わない。これは `core/mem.nepl` が低水準 byte operation と型付き所有権 operation の責務を同じ API に混ぜていることが根本原因である。

## 影響

collection、diagnostic、self-host AST node、`Result` payload などが owning value になるほど、同じ storage owner を複数値として保持できる。結果として double drop、use-after-free、stale alias、または leak が起きる。compiler の move/drop 検査が正しくても、typed bulk copy が byte layer へ逃がすため安全性の前提が崩れる。

## 修正方針

typed `mem_copy<T>` は `T: Copy` に限定する。non-Copy value の bulk move は、`OwnedRegion<T>` / `InitializedCell<T>` のような owner token と compiler Resource IR が source/destination の initialized state を更新する専用 API として設計する。既存の raw `mem_copy` / `mem_move` は unsafe/internal boundary に閉じ、public safe API からは直接使わせない。

## 検証

`MemPtr<Vec<i32>>` や Drop payload を持つ owner 型に対する typed `mem_copy` / `mem_move` を compile_fail にする。`u8` / `i32` / Copy struct の copy は成功させる。将来 owner-aware move API を入れる場合は、source が moved/uninitialized になり、destination が exactly-once drop obligation を持つことを Resource IR dump と compile_fail で確認する。
