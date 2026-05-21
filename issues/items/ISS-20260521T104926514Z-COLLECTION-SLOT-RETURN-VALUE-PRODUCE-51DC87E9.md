---
id: ISS-20260521T104926514Z-COLLECTION-SLOT-RETURN-VALUE-PRODUCE-51DC87E9
title: "Collection slot return value producer tracing must ignore never arms"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_path_value.rs
---

# ISS-20260521T104926514Z-COLLECTION-SLOT-RETURN-VALUE-PRODUCE-51DC87E9: Collection slot return value producer tracing must ignore never arms

## 概要

Return path state evaluation skips never-valued branch and match arms, but return value producer tracing still descends into then_value/else_value/arm.value without checking Never. This can derive return transfers or return slots from unreachable branch or match values.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_path_value.rs`

## 根拠

- `collection_slot_summary_return_path_state.rs` は Branch / Match の selected return path を作るとき `Never` value の arm を除外していた。
- 一方、`collection_slot_summary_return_path_value.rs` と legacy fallback の `collection_slot_summary_return_value.rs` は return value producer を逆追跡するときに `Never` value を除外せず、producer の root/projection が parameter と一致すれば return transfer を作り得た。
- return path state と return value producer tracing は同じ feasible path model に従う必要がある。

## 問題

Return path state evaluation skips never-valued branch and match arms, but return value producer tracing still descends into then_value/else_value/arm.value without checking Never. This can derive return transfers or return slots from unreachable branch or match values.

## 影響

Caller-side collection slot return summary replay can receive live slot state from an impossible path, producing false MaybeInitialized/LiveSlot diagnostics or hiding the actual feasible return path shape.

## 修正方針

Share an explicit Never-place predicate with return path control and use it in both path-sensitive and legacy return value producer tracing for Branch and Match producers.

## 対応

- return path control の `Never` 判定を `return_value_is_never` として共有し、Branch / Match path-state selection と return value producer tracing の両方から使うようにした。
- path-sensitive collector は `then_value` / `else_value` / `arm.value` が `Never` の場合、nested return path tracing を開始しない。
- legacy flat return-transfer collector も同じ判定を使い、`return_paths` が空の fallback 経路に古い unsound tracing が残らないようにした。
- regression として、`Never` value が parameter と同じ root/projection を持っていても return owner transfer として扱わず、caller の returned storage dealloc が impossible live slot を見ないことを Branch / Match で固定した。

## 検証

Add Resource IR regressions where a branch/match output is returned and only the never-valued producer path carries a live slot; caller storage dealloc must not see that impossible slot.

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_value_producer_skips_never_branch_value -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_value_producer_skips_never_match_value -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary_skips_never_branch_path_effects -- --test-threads=1`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_return_summary_skips_never_match_arm_path_effects -- --test-threads=1`: pass
- `cargo check -p nepl-core`: pass
- `cargo fmt --check`: pass
