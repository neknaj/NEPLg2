---
id: ISS-20260507T124325905Z-RESOURCE-OWNER-SUMMARY-MISSES-STRUCT-D34092E5
title: "Resource owner summary misses structured i32 raw owner projections"
area: resource
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-07
updated: 2026-05-07
target: nepl-core/src/resource/owner_summary_leaf.rs
---

# ISS-20260507T124325905Z-RESOURCE-OWNER-SUMMARY-MISSES-STRUCT-D34092E5: Resource owner summary misses structured i32 raw owner projections

## 概要

Owner summary leaf selection kept bare i32 non-owning, but also omitted i32 raw-owner slots nested under structs, tuples, and enum payload aggregates. ResourceIR then lost owners returned through Result<Boxed> match binds, aggregate field helpers, and function-value calls, leaving caller-side leaks.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir -- --nocapture` が 217 passed / 11 failed の状態で、残件が owner summary 系に集中していた。
- `resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind` は `Result::Ok(Boxed { ptr })` の `ptr` owner を `unwrap_box` の match bind から caller return へ移せず leak していた。
- `resource_ir_owner_check_consumes_only_used_aggregate_owner_projection` は `Pair.left` を callee で消費し `Pair.right` を caller に返す summary を表現できず、projection 単位の consume / return が崩れていた。
- `resource_ir_owner_check_transfers_owner_returned_by_function_value` は既に廃止済みの「裸 `i32` identity helper が raw owner を暗黙転送する」設計へ戻る stale regression だった。

## 問題

Owner summary leaf selection kept bare i32 non-owning, but also omitted i32 raw-owner slots nested under structs, tuples, and enum payload aggregates. ResourceIR then lost owners returned through Result<Boxed> match binds, aggregate field helpers, and function-value calls, leaving caller-side leaks.

## 影響

Memory-safety ResourceIR checks cannot prove one-shot transfer/deallocation of raw owners carried inside structured values. The full resource_ir regression suite remains noisy and can hide real static-check regressions.

## 修正方針

Keep root bare i32 non-owning, but seed structured i32 leaf projections so summaries can transfer only caller actuals that already have owner state. Update stale function-value and pending-result reservation regressions to assert the current ResourceIR authority rather than resurrecting plain i32 identity ownership.

## 検証

Run focused ResourceIR owner summary regressions, false-positive scalar regressions, full cargo test -p nepl-core --test resource_ir, cargo fmt/check, and issue validation.

## 2026-05-07 Agent 1 fixed

根本原因は、scalar owner leaf tightening 後の `owner_summary_leaf.rs` が root の裸 `i32` だけでなく、struct / tuple / enum payload aggregate の内側にある `i32` raw-owner slot まで owner summary source から落としていたことだった。root の裸 `i32` を再び owner とみなすと ordinary scalar と raw pointer proof が混ざるため戻さず、構造化 projection の leaf だけを追加した。

修正:

- aggregate field leaf 収集で、通常 owner leaf が空かつ field 型が `i32` の場合だけ structured scalar owner leaf を追加するようにした。
- root の裸 `i32` parameter はこれまで通り owner seed しないため、ordinary scalar identity helper の誤転送は復活しない。
- Result payload / aggregate field / function value 経由の owner projection transfer を focused regression で固定した。
- function value の古い bare `i32` owner-transfer regression は、構造化 aggregate owner projection を indirect call で転送する regression へ置き換えた。
- pending `dealloc_ptr` / `realloc_ptr` refinement regression は、reserved owner への違反が address read 段階で報告される現在の ResourceIR authority に合わせ、`Read | Dealloc` の reserved use を検査するようにした。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_consumes_only_used_aggregate_owner_projection -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_aggregate_owner_returned_by_function_value -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_plain_i32_identity_as_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_does_not_treat_plain_i32_struct_fields_as_owners -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 228 passed / 0 failed
- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `trunk build --release`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-structured-owner-summary-after-rebase.json -j 1 --dist web/dist`: total=14, passed=14
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-structured-owner-summary-after-rebase.json -j 1 --dist web/dist`: total=110, passed=110
- `node nodesrc/issues.js check`: passed
