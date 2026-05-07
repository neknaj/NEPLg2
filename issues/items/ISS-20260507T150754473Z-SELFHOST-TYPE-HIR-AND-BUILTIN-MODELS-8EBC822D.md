---
id: ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D
title: "Selfhost type HIR and builtin models use invalid sentinels instead of typed absence"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/mono/mono.nepl, stdlib/neplg2/core/builtins/prelude.nepl"
---

# ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D: Selfhost type HIR and builtin models use invalid sentinels instead of typed absence

## 概要

Selfhost type, HIR, mono, and builtin signature records still encode absent or unassigned state with numeric invalid IDs and Error-kind placeholder payloads. SelfhostTypeId(-1), first_arg = -1, SelfhostHirExprId(-1), SelfhostHirChildRange(-1, 0), SelfhostMonoInstanceId(-1), and builtin arg slots filled with SelfhostTypeKind::Error make invalid state representable in ordinary records.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/mono/mono.nepl, stdlib/neplg2/core/builtins/prelude.nepl`

## 根拠

- 未記入

## 問題

Selfhost type, HIR, mono, and builtin signature records still encode absent or unassigned state with numeric invalid IDs and Error-kind placeholder payloads. SelfhostTypeId(-1), first_arg = -1, SelfhostHirExprId(-1), SelfhostHirChildRange(-1, 0), SelfhostMonoInstanceId(-1), and builtin arg slots filled with SelfhostTypeKind::Error make invalid state representable in ordinary records.

## 影響

S3/S4 type, HIR, resource, and monomorphize work will not get the intended static-check guarantees if absent state is stored as i32 sentinels or broad Error placeholders. Match exhaustiveness cannot prove which payload fields are valid for each variant, and later code may accidentally accept invalid indices as normal values.

## 修正方針

Redesign these models around typed absence and variant-specific payloads: use Option or explicit Empty/Range enums for optional IDs and ranges, store function and builtin arguments in typed ranges or small signature enums instead of fixed placeholder slots, and split HIR expression payloads so each expression kind exposes only the fields it owns. Do not keep numeric invalid IDs as public constructors.

## 検証

Add source-policy tests rejecting new _invalid -> -1 helpers, first_* = -1 empty ranges, and SelfhostTypeKind::Error placeholder builtin arguments. Add focused .n.md tests for empty ranges, function signatures, invalid lookup rejection, and match-based handling of each payload variant.
