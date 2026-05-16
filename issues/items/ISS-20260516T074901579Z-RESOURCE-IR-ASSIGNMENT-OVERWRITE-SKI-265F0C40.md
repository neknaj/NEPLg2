---
id: ISS-20260516T074901579Z-RESOURCE-IR-ASSIGNMENT-OVERWRITE-SKI-265F0C40
title: "Resource IR assignment overwrite skips partial drop for moved aggregate descendants"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/resource/initialized_drop_assignment.rs, nepl-core/src/resource/initialized_drop_requirement.rs, nepl-core/tests/drop.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260516T074901579Z-RESOURCE-IR-ASSIGNMENT-OVERWRITE-SKI-265F0C40: Resource IR assignment overwrite skips partial drop for moved aggregate descendants

## 概要

assignment overwrite drop recording only checks the whole target CellState::Initialized path. If an aggregate has one field moved out and another droppable field still initialized, overwriting the aggregate can skip the remaining field drop before replacement.

## 対象

- `nepl-core/src/resource/initialized_drop_assignment.rs, nepl-core/src/resource/initialized_drop_requirement.rs, nepl-core/tests/drop.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/resource/initialized_drop_scope.rs` は scope end で whole target が `Initialized` でなくても、initialized descendant を `partial_drop_requirement_for_initialized_descendants` で拾う。
- 一方 `nepl-core/src/resource/initialized_drop_assignment.rs` は assignment overwrite で whole target が `Initialized` の場合だけ drop fact を作っていた。
- そのため `Pair { left: GuardA, right: GuardB }` の `left` だけを move out した後に `set p Pair ...` すると、旧 `right` field の Drop obligation を上書き前に codegen plan へ渡せなかった。

## 問題

assignment overwrite drop recording only checks the whole target CellState::Initialized path. If an aggregate has one field moved out and another droppable field still initialized, overwriting the aggregate can skip the remaining field drop before replacement.

## 影響

A partially moved owning aggregate can lose a live field owner at assignment overwrite. This weakens Resource IR as the final drop authority and can leak memory/resource obligations while tests still pass for scope-end partial drops.

## 修正方針

Reuse the same partial initialized descendant drop requirement logic used by scope-end auto drop for assignment overwrite. Record a checked assignment overwrite drop fact whenever the whole target or any initialized droppable descendant requires drop, then mark the overwritten target dropped before assigning the replacement.

## 検証

Add Resource IR and runtime drop regressions for moving one field out of an aggregate and then overwriting the aggregate. Verify both the remaining old field and the new aggregate are dropped exactly once.

## 修正内容

- assignment overwrite の live drop fact 生成を scope end と同じ partial initialized descendant requirement 経路へ揃えた。
- whole target が `Initialized` の場合は従来通り target 全体の `ResourceDropRequirement` を使う。
- whole target が moved / partially moved でも、子孫 field に initialized Drop obligation が残っていれば `Structural` requirement として checked plan に記録する。
- initialized descendant がなければ `StateOnly` として drop fact を出さず、move 済み target の再初期化は従来通り余計な drop を挿入しない。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_assignment_overwrite_records_partial_drop_after_field_move -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test drop assignment_overwrite_drops_remaining_fields_after_partial_move -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir assignment_overwrite -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_drop_elaboration_plan_skips_moved_assignment_targets -- --exact --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_drop_insertion_consumes_checked_scope_and_assignment_points -- --exact --nocapture`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
