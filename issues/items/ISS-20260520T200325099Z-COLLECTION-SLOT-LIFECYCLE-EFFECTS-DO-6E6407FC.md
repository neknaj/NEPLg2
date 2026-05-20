---
id: ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC
title: "Collection slot lifecycle effects do not summarize across calls"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_summary_*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T200325099Z-COLLECTION-SLOT-LIFECYCLE-EFFECTS-DO-6E6407FC: Collection slot lifecycle effects do not summarize across calls

## 概要

ResourceOp::CollectionSlotLifecycle is checked inside a function, but ResourceOp::Call only applies raw-cell and scalar summaries. A callee that initializes, moves, drops, or releases collection slots cannot update the caller CollectionSlotStateTable before caller-side operations such as storage dealloc.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/initialized_summary_*.rs, nepl-core/src/resource/model.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

ResourceOp::CollectionSlotLifecycle is checked inside a function, but ResourceOp::Call only applies raw-cell and scalar summaries. A callee that initializes, moves, drops, or releases collection slots cannot update the caller CollectionSlotStateTable before caller-side operations such as storage dealloc.

## 影響

Non-Copy collection payload safety would remain intra-function only. Once stdlib collection APIs lower to collection slot lifecycle events, caller code could miss callee slot state transitions unless every lifecycle-changing call is inlined or handled by an unsafe module-specific convention.

## 修正方針

Design and implement a typed collection-slot lifecycle summary for Resource IR calls. The summary must be generic over Place suffixes and storage owner arguments, integrate with branch/loop/match path merges, and apply through ResourceOp::Call and IndirectCall without stdlib allowlists.

## 検証

cargo test -p nepl-core collection_slot_call_summary -- --test-threads=1; cargo test -p nepl-core resource_ir_collection_slot -- --test-threads=1; node nodesrc/test_resource_checker_responsibility.js; node nodesrc/issues.js check --dir issues; git diff --check
