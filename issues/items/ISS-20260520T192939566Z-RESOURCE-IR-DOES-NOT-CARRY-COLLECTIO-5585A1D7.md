---
id: ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7
title: "Resource IR does not carry collection slot lifecycle events"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/initialized.rs, nepl-core/src/resource/report.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T192939566Z-RESOURCE-IR-DOES-NOT-CARRY-COLLECTIO-5585A1D7: Resource IR does not carry collection slot lifecycle events

## 概要

CollectionSlotLifecycleEvent and CollectionSlotStateTable exist, but ResourceOp has no collection slot operation and the resource checker keeps no collection slot state. The compiler cannot apply the generic slot proof boundary to Resource IR control flow, so non-Copy collection payload safety remains outside the checked program model.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/initialized.rs, nepl-core/src/resource/report.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection payload を stdlib module の個別 allowlist ではなく Resource IR の generic proof boundary へ載せる方針を定めている。
- 既存の `CollectionSlotLifecycleEvent` / `CollectionSlotStateTable` は `Place` ごとの state と control-flow merge を表せていたが、`ResourceOp` と initialized checker がその状態を持っていなかったため、実際の Resource IR program に適用されなかった。
- `cargo check -p nepl-core` により `ResourceOp::CollectionSlotLifecycle` を追加した後の exhaustive `match` 漏れを洗い出し、borrow / owner / effect / summary / coverage / dump などの全 Resource IR consumer へ明示的に接続した。

## 問題

CollectionSlotLifecycleEvent and CollectionSlotStateTable exist, but ResourceOp has no collection slot operation and the resource checker keeps no collection slot state. The compiler cannot apply the generic slot proof boundary to Resource IR control flow, so non-Copy collection payload safety remains outside the checked program model.

## 影響

Non-Copy collection support can only be implemented by stdlib/module-specific allowlists or unchecked raw memory conventions. Branch/match/loop slot merge state is unreachable from actual Resource IR checking, so partial move/drop/release bugs would not be rejected by the compiler.

## 修正方針

Add a ResourceOp variant carrying a slot Place and CollectionSlotLifecycleEvent, thread CollectionSlotStateTable through the initialized/resource checker including branch/loop/match merges, preserve CollectionSlotTableRefutation as a typed diagnostic, and add Resource IR regression tests for double move and partial path liveness.

## 修正内容

- `ResourceOp::CollectionSlotLifecycle { target, event, span }` を追加し、slot または storage target と `CollectionSlotLifecycleEvent` を Resource IR の通常命令として表すようにした。
- initialized checker は `CollectionSlotStateTable` を関数チェック状態として保持し、branch / loop / match の各 path で clone して `CollectionSlotStateTable::merge_paths` により合流する。
- `CollectionSlotTableRefutation` は `ResourceCheckDiagnostic::CollectionSlotRefuted` として typed diagnostic に保持し、`ResourceCollectionSlotDiagnosticCode` の階層 enum へ写像する。文字列 diagnostic id や sentinel は追加していない。
- Resource IR の coverage / dump / borrow / owner / effect / summary walker は `ResourceOp::CollectionSlotLifecycle` を exhaustive `match` で明示処理する。
- 回帰テストとして、同一 slot の二重 move-out を拒否するテストと、branch merge 後に片側だけ live な slot の storage dealloc を拒否するテストを追加した。

## 検証

- `cargo test -p nepl-core resource_ir_collection_slot -- --test-threads=1`
- `cargo test -p nepl-core collection_slot -- --test-threads=1`
- `cargo check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
