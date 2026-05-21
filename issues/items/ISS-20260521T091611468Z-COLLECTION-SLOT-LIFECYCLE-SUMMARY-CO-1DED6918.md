---
id: ISS-20260521T091611468Z-COLLECTION-SLOT-LIFECYCLE-SUMMARY-CO-1DED6918
title: "Collection slot lifecycle summary collection ignores match-bound enum payload state"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_build_ops.rs, nepl-core/src/resource/collection_slot_summary_return_collect.rs"
---

# ISS-20260521T091611468Z-COLLECTION-SLOT-LIFECYCLE-SUMMARY-CO-1DED6918: Collection slot lifecycle summary collection ignores match-bound enum payload state

## 概要

Collection slot lifecycle summary collection recurses into ResourceOp::Match arms without constructing the per-arm entry state for owned enum payload bind locals. Lifecycle events inside a match arm can therefore fail to connect a bind local back to the parameter payload and may be omitted from the generic summary proof.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_ops.rs, nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- `collection_slot_summary_build_ops.rs` の `ResourceOp::Match` summary 収集は、arm の `ops` を pre-match state のまま `collect_nested_summary_ops` に渡していた。
- 一方、return-transfer 収集と本体 initialized checker は、owned enum payload bind local に対して `scrutinee.EnumPayload(variant)` から bind local へ raw alias、cell origin、collection slot state、function alias、pending state を伝播してから arm を検査する。
- この差分により、callee が `match result` の `Err storage` arm 内で `CollectionSlotLifecycle::StorageDealloc` を発行しても、summary event の target が parameter payload に戻らず、caller summary replay が lifecycle effect を見落とし得た。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、`Result` や stdlib 関数名の個別許可ではなく、Resource IR の typed state / summary proof によって性質を証明する方針を要求している。

## 問題

Collection slot lifecycle summary collection recurses into ResourceOp::Match arms without constructing the per-arm entry state for owned enum payload bind locals. Lifecycle events inside a match arm can therefore fail to connect a bind local back to the parameter payload and may be omitted from the generic summary proof.

## 影響

Compiler-owned collection cleanup through Result/Option-like enum payloads can lose callee-certified lifecycle effects, allowing caller replay to miss live-slot storage deallocation or other non-Copy payload state transitions.

## 修正方針

Share the same generic match arm entry-state construction used by return-transfer collection and the initialized checker, then collect arm summary ops against that state instead of the pre-match state.

## 修正内容

- `collection_slot_summary_match_state.rs` を追加し、collection slot summary が使う match arm entry-state 構築を return-transfer 専用 private 関数から共通 helper へ切り出した。
- summary event 収集側の `ResourceOp::Match` は、各 reachable arm について owned enum payload bind local の raw alias / cell origin / slot state / function alias / pending realloc / variant state を entry state に反映してから nested summary ops を収集するようにした。
- return-transfer 収集側も同じ helper を使うようにし、summary event と return-transfer の match semantics が分岐しないようにした。
- `Result`、`Option`、stdlib module 名、関数名の allowlist は追加していない。`ResourceOp::Match` と `ResourceMatchArm` の typed state から generic に導出する。

## 検証

Add a Resource IR regression where a callee matches an enum parameter and emits CollectionSlotLifecycle::StorageDealloc on the owned payload bind local; the caller must receive the summary and report LiveSlotDuringStorageDealloc.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_collects_match_bound_lifecycle_event -- --test-threads=1`: passed
