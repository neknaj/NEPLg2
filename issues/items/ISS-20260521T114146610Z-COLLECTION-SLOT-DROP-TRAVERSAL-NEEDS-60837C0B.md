---
id: ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B
title: "Collection slot drop traversal needs generic range proof"
area: compiler
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/**"
---

# ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B: Collection slot drop traversal needs generic range proof

## 概要

Collection cleanup for non-Copy payloads has single-slot DropInitialized proof, but there is no generic Resource IR proof that a collection-wide traversal dropped every initialized slot in a dynamic or symbolic range before StorageDealloc.

## 対象

- `nepl-core/src/resource/**`

## 根拠

- subagent review で、single-slot `DropInitialized` / `DropLoadedCell` proof は存在する一方、`Vec<T>` / `OwnedBuffer<T>` などの collection cleanup が storage 配下の initialized non-Copy slot 全体を走査して drop 済みにしたことを表す汎用 proof boundary がまだ存在しないことを確認した。
- 現在の `StorageDealloc` は live slot が残っていれば拒否できるが、dynamic len / symbolic offset / loop traversal に対して「対象範囲の全 initialized slot を実際に drop した」という positive proof はまだ表現できない。
- `DropInitialized` の単発 proof を loop 全体の proof として流用したり、stdlib の `vec_free` / `hashmap_free` などの関数名 allowlist で cleanup 済み扱いにしたりすると、静的検査の正確性と generic proof solver 方針に反する。
- 関連計画: [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 問題

Collection cleanup for non-Copy payloads has single-slot DropInitialized proof, but there is no generic Resource IR proof that a collection-wide traversal dropped every initialized slot in a dynamic or symbolic range before StorageDealloc.

## 影響

Self-host Vec/OwnedBuffer cleanup could either stay Copy-only or reintroduce module-specific allowlists/cleanup booleans. That would leave non-Copy collection drop/free without a source-derived proof that all initialized cells were actually dropped.

## 修正方針

Design and implement a typed collection slot traversal proof that derives all-slot cleanup from Resource IR facts, consumes loaded-value drop proof per initialized slot, preserves partial traversal as MaybeInitialized across path merge, and carries certified traversal evidence through summaries.

実装では次を満たす。

- traversal proof は collection module ごとの個別証明器ではなく、Resource IR の fact / obligation / evidence / refutation に載せる。
- partial traversal や片 branch だけの cleanup は `MaybeInitialized` として merge 後に残し、`StorageDealloc` を通さない。
- raw load だけでは drop proof としない。各 initialized slot について actual loaded-value drop proof を消費する。
- callee で証明した traversal cleanup は summary に typed evidence として載せ、caller replay で stdlib allowlist なしに再生する。

## 検証

Add regressions for full traversal allowing storage dealloc, partial traversal rejection, raw load without actual drop rejection, and summary replay without stdlib allowlists.
