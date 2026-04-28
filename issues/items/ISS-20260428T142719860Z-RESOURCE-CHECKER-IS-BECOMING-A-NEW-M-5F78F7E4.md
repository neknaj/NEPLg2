---
id: ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4
title: "Resource checker is becoming a new monolithic static-check pass"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/src/resource/effect.rs, nepl-core/src/resource/mod.rs"
---

# ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4: Resource checker is becoming a new monolithic static-check pass

## 概要

The static-check migration split typecheck.rs and move_check.rs, but Resource IR enforcement is now accumulating cell state, owner obligation, borrow lifetime, effect boundary, summaries, merge helpers, and raw memory cell utilities inside nepl-core/src/resource/check.rs. The file is already 2674 lines after Stage 4 raw storage fixes.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/src/resource/effect.rs, nepl-core/src/resource/mod.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 2026-04-28 の Stage 4 raw storage cell 修正後、`nepl-core/src/resource/check.rs` は 2674 行になっている。
- 同じ時点で `nepl-core/src/resource/effect.rs` は 1197 行、`nepl-core/src/resource/lower.rs` は 731 行であり、Resource IR 周辺の責務はすでに分離対象として十分大きい。
- `check.rs` には `ResourceCheckEngine`、`ResourceOwnerCheckEngine`、`ResourceBorrowCheckEngine`、function owner summary、borrow table、owner table、cell table、raw cell helper、merge helper、shadow report が同居している。

## 問題

The static-check migration split typecheck.rs and move_check.rs, but Resource IR enforcement is now accumulating cell state, owner obligation, borrow lifetime, effect boundary, summaries, merge helpers, and raw memory cell utilities inside nepl-core/src/resource/check.rs. The file is already 2674 lines after Stage 4 raw storage fixes.

## 影響

If Stage 4/5 keeps adding checks in one file, the project will recreate the same ad-hoc responsibility concentration that the Resource IR migration is intended to remove. Authoritative Resource IR gating will become harder to audit, and future self-host work will depend on another oversized static-check pass.

## 修正方針

Split resource checking by responsibility before enabling broader authoritative gates: cell_state, owner_obligation, borrow_lifetime, raw_cell, function_summary, state_merge, and shadow_report modules. Keep public exports stable through resource/mod.rs and move tests only when behavior is unchanged.

## 検証

After splitting, run the full resource_ir test suite, rustfmt on the resource modules, node nodesrc/issues.js check, and trunk build. Add a source-level check or documentation note so Resource IR check responsibilities do not collapse back into one file.
