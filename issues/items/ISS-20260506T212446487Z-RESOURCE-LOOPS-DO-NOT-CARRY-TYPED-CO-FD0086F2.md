---
id: ISS-20260506T212446487Z-RESOURCE-LOOPS-DO-NOT-CARRY-TYPED-CO-FD0086F2
title: "Resource loops do not carry typed condition facts into body paths"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/owner_control.rs"
---

# ISS-20260506T212446487Z-RESOURCE-LOOPS-DO-NOT-CARRY-TYPED-CO-FD0086F2: Resource loops do not carry typed condition facts into body paths

## 概要

ResourceOp::Branch carries condition_fact, but ResourceOp::Loop stores only condition_ops, condition, and body_ops. As a result, loop body checking cannot record typed facts such as i < len into RawCellAddressAliases, so guarded dynamic stores/loads inside loops cannot contribute to initialized range summaries.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/owner_control.rs`

## 根拠

- `ResourceOp::Branch` は `condition_fact` を持ち、lowering / dump / owner checker で typed fact を扱っていた。
- 一方で `ResourceOp::Loop` は `condition_ops` / `condition` / `body_ops` だけを持ち、`while lt i len` の fact が Resource IR から消えていた。
- initialized checker と owner checker は loop body path state を condition evaluation 後に clone するため、この地点で truthy fact を反映しないと body 側の `RawCellAddressAliases` から relation proof を参照できない。
- guarded initialized range summary は HIR 条件式の再走査ではなく Resource IR state を authority にする必要があるため、Loop 自体が typed fact を保持する必要がある。

## 問題

ResourceOp::Branch carries condition_fact, but ResourceOp::Loop stores only condition_ops, condition, and body_ops. As a result, loop body checking cannot record typed facts such as i < len into RawCellAddressAliases, so guarded dynamic stores/loads inside loops cannot contribute to initialized range summaries.

## 影響

Even after I32Relation and relation fact stores exist, loops lose the proof at the only point where induction-style buffer writes are checked. Future guarded range summaries would either miss loop writes or need to re-read HIR conditions, weakening the Resource IR authority boundary.

## 修正方針

Add condition_fact to ResourceOp::Loop, lower it from the loop condition with the same typed condition fact logic used for branches, dump it for auditability, and apply the truthy fact to loop body path state in initialized and owner checking.

## 検証

- `cargo fmt --check -p nepl-core`
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_loop_i32_relation_condition_fact -- --nocapture`
- `cargo test -p nepl-core initialized_branch_condition_fact_records -- --nocapture`
- `cargo check -p nepl-core --tests`
- `node nodesrc/issues.js check`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `git diff --check`

## 2026-05-07 対応結果

`ResourceOp::Loop` に `condition_fact: Option<ResourceConditionFact>` を追加し、`HirExprKind::While` lowering で branch と同じ `resource_condition_fact` を使って typed condition fact を保存するようにした。Resource IR dump も `loop cond=... fact=...` を出すため、loop guard が audit 可能になった。

initialized checker と owner checker の loop handling は、condition evaluation 後に exit path へ false fact、body path へ truthy fact を適用してから body ops を検査する。これにより `while lt i len` の body では `i < len`、loop exit では `i >= len` が `RawCellAddressAliases` に残り、後続の guarded range summary が HIR を再読せずに Resource IR state へ問い合わせられる。

coverage/dump/borrow/effect/drop/summary の Loop pattern は新 field を明示的に扱うか無視する形へ更新した。これは condition fact を暗黙に落とさず、今後 Loop model を変えた場合に Rust の pattern exhaustiveness で追跡できるようにするためである。

回帰として `resource_ir_lowering_preserves_loop_i32_relation_condition_fact` を追加し、`while lt i len` が `ResourceConditionFact::I32Relation { op: Lt }` として lowered IR と dump に残ることを固定した。
