---
id: ISS-20260522T224620549Z-OWNER-SUMMARY-MUST-SEED-OWNER-TOKEN--89E3E5BE
title: "Owner summary must seed owner-token leaves for higher-order consumption"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260522T224620549Z-OWNER-SUMMARY-MUST-SEED-OWNER-TOKEN--89E3E5BE: Owner summary must seed owner-token leaves for higher-order consumption

## 概要

Owner return summaries did not seed RegionToken raw owner leaves as ordinary owner leaves. A generic eliminator that moves an owner out of an aggregate and passes it to an indirect callback could therefore produce an empty summary and leave the caller-side owner live.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

Owner return summaries did not seed RegionToken raw owner leaves as ordinary owner leaves. A generic eliminator that moves an owner out of an aggregate and passes it to an indirect callback could therefore produce an empty summary and leave the caller-side owner live.

## 影響

Higher-order owner-preserving APIs such as VecPop eliminators can be rejected with resource.owner.leak even though the source consumes the owner through a callback, blocking non-Copy collection move-out APIs and self-host data-structure patterns.

## 修正方針

Treat compiler-proven owner-token raw fields as owner leaf projections during summary parameter seeding, then verify higher-order aggregate decomposition through Resource IR owner obligation tests.

## 検証

Run owner_summary_leaf unit tests and focused Resource IR Vec<DropPayload>.pop owner-obligation regression.

## 2026-05-22 Agent 1 修正

`owner_summary_leaf` が compiler-proven `RegionToken<T>` / owner-token raw field を owner leaf として seed するようにした。従来は raw owner usage が既に検出できる場合だけ raw i32 leaf を後追いで seed していたため、owner を aggregate から取り出して unknown indirect callback に渡す generic eliminator では、summary engine の入力に owner state が存在しなかった。

修正後は owner token field name / compiler memory type evidence に基づく `owner_token_raw_i32_leaf_projections` を `owner_leaf_projections_mapped` の入口で扱う。これにより by-value owner token と、それを含む aggregate parameter は最初から free obligation owner として summary engine に載る。stdlib 関数名や module allowlist は追加していない。

回帰テストとして owner token raw field が owner leaf に seed される unit test と、`VecPop<T>` eliminator の callback 境界を越えて owner obligation が閉じる Resource IR test を追加した。

検証:

- `cargo test -p nepl-core resource::owner_summary_leaf::tests::owner_leaf_places_seed_owner_token_raw_field -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_pop_moves_out_tail_slot_and_recovers_owners -- --test-threads=1 --exact --nocapture`
