---
id: ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC
title: "Collection slot lifecycle effects do not summarize across calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_summary_*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC: Collection slot lifecycle effects do not summarize across calls

## 概要

ResourceOp::CollectionSlotLifecycle is checked inside a function, but ResourceOp::Call only applies raw-cell and scalar summaries. A callee that initializes, moves, drops, or releases collection slots cannot update the caller CollectionSlotStateTable before caller-side operations such as storage dealloc.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_summary_*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy collection slot lifecycle を stdlib module allowlist ではなく Resource IR の generic typed proof boundary に載せる方針を定めている。
- `ResourceOp::Call` / `ResourceOp::IndirectCall` は raw cell initialization summary と i32 scalar summary だけを caller へ適用しており、`CollectionSlotStateTable` は関数内の state merge に閉じていた。
- `ResourceOp::CollectionSlotLifecycle` を実際の collection lowering へ接続する前に、callee 内 lifecycle effect が caller の slot state へ伝播しなければ、helper 関数を跨いだ storage dealloc / move / drop の安全性を証明できない。

## 問題

ResourceOp::CollectionSlotLifecycle is checked inside a function, but ResourceOp::Call only applies raw-cell and scalar summaries. A callee that initializes, moves, drops, or releases collection slots cannot update the caller CollectionSlotStateTable before caller-side operations such as storage dealloc.

## 影響

Non-Copy collection payload safety would remain intra-function only. Once stdlib collection APIs lower to collection slot lifecycle events, caller code could miss callee slot state transitions unless every lifecycle-changing call is inlined or handled by an unsafe module-specific convention.

## 修正方針

Design and implement a typed collection-slot lifecycle summary for Resource IR calls. The summary must be generic over Place suffixes and storage owner arguments, integrate with branch/loop/match path merges, and apply through ResourceOp::Call and IndirectCall without stdlib allowlists.

## 修正内容

- `CollectionSlotLifecycleFunctionSummary` を追加し、callee parameter projection 上の `CollectionSlotLifecycleEvent` を型付き summary program として保持するようにした。
- summary は `Event` / `Merge` / `Loop` の enum で表し、branch / match / indirect-call alternatives は path merge、loop は condition path と body path の merge として caller 側の `CollectionSlotStateTable` に再適用する。
- direct call は callee summary を actual arg の suffix に置換して適用し、indirect call は `FunctionAliasTable` から得た候補関数ごとに path を clone して merge する。stdlib module 名や関数名の allowlist は追加していない。
- initialized checker の direct / indirect call 処理を `initialized_call.rs` に分割し、summary setup 用の `initialized_summary_engine.rs` を共有化して責務分割 line policy を維持した。
- 回帰テストとして、callee が slot を initialize+move する場合の caller storage dealloc 許可、callee が live slot を残す場合の拒否、callee branch の partial live state の `MaybeLiveSlotDuringStorageDealloc`、function alias indirect call の summary 適用を追加した。

## 検証

cargo test -p nepl-core collection_slot_call_summary -- --test-threads=1; cargo test -p nepl-core resource_ir_collection_slot -- --test-threads=1; node nodesrc/test_resource_checker_responsibility.js; node nodesrc/issues.js check --dir issues; git diff --check
