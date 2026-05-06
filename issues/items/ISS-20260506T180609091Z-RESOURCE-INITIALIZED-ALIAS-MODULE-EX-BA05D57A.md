---
id: ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A
title: "Resource initialized alias module exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_alias.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260506T180609091Z-RESOURCE-INITIALIZED-ALIAS-MODULE-EX-BA05D57A: Resource initialized alias module exceeds responsibility split limit

## 概要

After splitting lower_raw_address.rs, node nodesrc/test_resource_checker_responsibility.js reaches the next responsibility guard failure: initialized_alias.rs has 624 lines while the split limit is 520.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1` の対応で `lower_raw_address.rs` を 620 行未満へ分割した後、`node nodesrc/test_resource_checker_responsibility.js` が次の blocker として本 issue を報告した。
- 失敗内容は `initialized_alias.rs has 624 lines; responsibility split limit is 520`。
- `initialized_alias.rs` は raw address alias group、stable value origin、i32 value/condition fact、branch merge、prefix/projected alias utility を同居させており、Resource IR cell / owner の両方に関わるため分割して監査可能にする必要がある。

## 問題

After splitting lower_raw_address.rs, node nodesrc/test_resource_checker_responsibility.js reaches the next responsibility guard failure: initialized_alias.rs has 624 lines while the split limit is 520.

## 影響

Raw address alias tracking is a memory-safety-critical support module for initialized cell and owner checks. Keeping value origin, i32 facts, alias groups, merge, and projection utilities in one large file makes future static-check changes harder to audit.

## 修正方針

Split initialized_alias.rs by semantic responsibility instead of raising the limit. Keep RawCellAddressAliases orchestration in initialized_alias.rs and move value-origin resolution, i32 scalar facts, or projected alias utilities into focused modules with policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only.

## 2026-05-07 修正

`initialized_alias.rs` から stable value origin と i32 scalar fact store を分離した。

分離後の責務は次の通り。

- `initialized_alias.rs`: raw address alias group、marked raw owner cell、canonicalization、prefix/projected alias query、path merge orchestration。
- `initialized_alias_origin.rs`: stable local / return / storage origin の追跡、origin suffix 解決、branch path 共通 origin merge。
- `initialized_alias_scalar.rs`: i32 value fact / condition fact の copy、clear、branch path 共通 fact merge、condition implication query。
- `initialized_alias_i32.rs`: i32 fact data と condition implication の enum-first rule。

この分割により、memory-safety-critical な raw owner alias group と、value-origin / condition fact の補助状態が同一 file に再集中しない構造になった。`nodesrc/test_resource_checker_responsibility.js` には新 module の存在、`mod` 宣言、entry point、line limit を追加した。

検証:

- `cargo fmt --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_refines_zero_alloc_result_branch -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_merge_rejects_dealloc_after_conditional_dealloc -- --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: all source-policy regressions passed; warning 0
