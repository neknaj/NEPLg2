---
id: ISS-20260521T114146610Z-COLLECTION-SLOT-DROP-TRAVERSAL-NEEDS-60837C0B
title: "Collection slot drop traversal needs generic range proof"
area: compiler
status: fixed
resolved: true
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

## 対応内容

- `ResourceOp::CollectionSlotDropTraversal` と `CollectionSlotLifecycleOp::DropTraversal` を追加し、storage prefix 配下の initialized collection slot を列挙して、各 slot に `DropInitialized` の loaded-value drop proof を要求する汎用 traversal proof boundary を設けた。
- traversal は `CollectionSlotStateTable` と `CellTable` の clone 上で全 slot の proof を検査し、すべて成功した場合だけ commit する。途中で `DropRequiresElaboration` や `MaybeLiveSlotDuringStorageDealloc` が出た場合、先に検査した slot の proof 消費や state mutation は残さない。
- raw load だけでは cleanup proof とせず、actual `Drop` から得た `LocalLoadedValueDrop` proof がなければ traversal を拒否する。summary replay では callee 側で証明済みの traversal だけを `SummaryCertified` として再生する。
- call summary に `CollectionSlotLifecycleSummaryOp::DropTraversal` と `CollectionSlotLifecycleSummaryDropTraversalProof::CertifiedLoadedValueDrops` を追加し、callee の certified traversal を caller actual storage へ instantiate して replay する。non-Copy raw cell については summary replay 時にも raw cell moved state を進め、後続 raw dealloc / storage dealloc と矛盾しないようにした。
- coverage / dump / effect / borrow / owner / initialized flow / summary dependency の各 `ResourceOp` match に明示 arm を追加し、新しい traversal op の取り扱い漏れを Rust の網羅性検査で検出できる形にした。
- `initialized_collection_slot_dispatch.rs` を追加し、collection slot 系 ResourceOp の initialized checker dispatch を本体から分けて `initialized.rs` の責務肥大化を避けた。
- `nodesrc/test_resource_checker_responsibility.js` の Resource checker 責務監視表に traversal module を登録し、同じ検査で露見した storage release proof module の監視漏れと Stage 6 collection-slot 関連 module の古い行数上限を現在の分割単位へ再同期した。

## 検証

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir collection_slot_drop_traversal -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- 追加した回帰:
  - full traversal が全 initialized slot の loaded-value drop proof を消費した後だけ raw dealloc / storage dealloc を通す。
  - summary replay が certified traversal と non-Copy raw cell moved state を caller 側に再生する。
  - 一部 slot の drop proof 欠落は atomic に拒否し、先行 slot の state / proof 消費を残さない。
  - raw load だけでは drop traversal proof にならない。
  - 片 branch だけの cleanup は merge 後に maybe-live として残り、storage dealloc を証明しない。
