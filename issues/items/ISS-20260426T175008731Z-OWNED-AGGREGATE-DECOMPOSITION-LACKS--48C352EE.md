---
id: ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE
title: "owned aggregate decomposition lacks safe multi-field move path"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, stdlib/alloc/diag/error.nepl, stdlib/neplg2/core/infra/outcome.nepl"
---

# ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE: owned aggregate decomposition lacks safe multi-field move path

## 概要

When an owning struct contains multiple non-Copy fields, helper code cannot move out more than one field with field::get because the owner is considered moved after the first non-Copy field extraction. Existing stdlib code works around similar cases with raw memory store/load detours.

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, stdlib/alloc/diag/error.nepl, stdlib/neplg2/core/infra/outcome.nepl`

## 根拠

- 未記入

## 問題

When an owning struct contains multiple non-Copy fields, helper code cannot move out more than one field with field::get because the owner is considered moved after the first non-Copy field extraction. Existing stdlib code works around similar cases with raw memory store/load detours.

## 影響

Outcome-like values that need to return a Result and free or propagate diagnostics are pushed toward raw memory detours or indirect pointer layouts. This makes ownership intent harder to audit and can hide real move/borrow bugs from stdlib review.

## 修正方針

Design a safe owned aggregate decomposition path, such as compiler-supported struct destructuring or a checked multi-field move primitive, so code can consume an owner and bind all fields exactly once without raw memory round-trips.

## 検証

Add compiler tests that consume a struct with two non-Copy fields and bind both fields once, while still rejecting repeated moves, partial use-after-move, and borrow-live owner moves.
