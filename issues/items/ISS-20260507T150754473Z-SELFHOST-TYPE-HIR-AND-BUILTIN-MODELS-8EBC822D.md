---
id: ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D
title: "Selfhost typed IR models use invalid sentinels instead of typed absence"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/mono/mono.nepl, stdlib/neplg2/core/builtins/prelude.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl"
---

# ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D: Selfhost typed IR models use invalid sentinels instead of typed absence

## 概要

Selfhost resolver, type, HIR, mono, and builtin signature records still encode absent or unassigned state with numeric invalid IDs, numeric enum tags, and Error-kind placeholder payloads. SelfhostDefId(-1), SelfhostTypeId(-1), first_arg = -1, SelfhostHirExprId(-1), SelfhostHirChildRange(-1, 0), SelfhostMonoInstanceId(-1), SelfhostDefKind -> i32 comparison tags, and builtin arg slots filled with SelfhostTypeKind::Error make invalid or non-exhaustive state representable in ordinary records.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl, stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/mono/mono.nepl, stdlib/neplg2/core/builtins/prelude.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl`

## 根拠

- 未記入

## 問題

Selfhost resolver, type, HIR, mono, and builtin signature records still encode absent or unassigned state with numeric invalid IDs, numeric enum tags, and Error-kind placeholder payloads. SelfhostDefId(-1), SelfhostTypeId(-1), first_arg = -1, SelfhostHirExprId(-1), SelfhostHirChildRange(-1, 0), SelfhostMonoInstanceId(-1), SelfhostDefKind -> i32 comparison tags, and builtin arg slots filled with SelfhostTypeKind::Error make invalid or non-exhaustive state representable in ordinary records.

## 影響

S3/S4 resolve, type, HIR, resource, and monomorphize work will not get the intended static-check guarantees if absent state is stored as i32 sentinels, broad Error placeholders, or enum-to-number tags. Match exhaustiveness cannot prove which payload fields are valid for each variant, and later code may accidentally accept invalid indices or stale comparison tags as normal values.

## 修正方針

Redesign these models around typed absence and variant-specific payloads: use Option or explicit Empty/Range enums for optional IDs and ranges, store function and builtin arguments in typed ranges or small signature enums instead of fixed placeholder slots, and split HIR expression payloads so each expression kind exposes only the fields it owns. Compare enums through direct match coverage or dedicated enum-equality helpers that do not expose numeric tags. Do not keep numeric invalid IDs as public constructors.

## 検証

Add source-policy tests rejecting new _invalid -> -1 helpers, first_* = -1 empty ranges, enum-to-i32 comparison tags, and SelfhostTypeKind::Error placeholder builtin arguments. Add focused .n.md tests for empty ranges, function signatures, invalid lookup rejection, and match-based handling of each payload variant.

## 2026-05-08 enum equality tag helper 対応

この親 issue のうち、`SelfhostTypeKind` / `SelfhostHirExprKind` / `SelfhostBuiltinKind` / `SelfhostDefKind` の equality helper が enum を i32 tag に落として比較していた問題は、[ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87](./ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87.md) で分離して解決した。

残件:

- `SelfhostDefId(-1)` / `SelfhostHirExprId(-1)` / `SelfhostMonoInstanceId(-1)` の invalid sentinel。
- `SelfhostHirChildRange(-1, 0)`、`SelfhostHirParamRange(-1, 0)` の empty range sentinel。
- HIR expression payload が kind ごとに所有 field を分離できていないこと。

## 2026-05-08 builtin signature payload 対応

この親 issue のうち、`SelfhostBuiltinFunction` が固定 `arg0` / `arg1` / `arg2` slot と `arg_count` で signature を表し、未使用 slot に `SelfhostTypeKind::Error` を入れていた問題は、[ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4](./ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4.md) で分離して解決した。

残件:

- `SelfhostDefId(-1)` / `SelfhostHirExprId(-1)` / `SelfhostMonoInstanceId(-1)` の invalid sentinel。
- `SelfhostHirChildRange(-1, 0)`、`SelfhostHirParamRange(-1, 0)` の empty range sentinel。
- HIR expression payload が kind ごとに所有 field を分離できていないこと。

## 2026-05-08 type record payload 対応

この親 issue のうち、`SelfhostTypeRecord` が primitive/function 共通の flat field を持ち、primitive record に `first_arg = -1` と invalid `TypeId` を入れていた問題は、[ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D](./ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D.md) で分離して解決した。

残件:

- `SelfhostDefId(-1)` / `SelfhostHirExprId(-1)` / `SelfhostMonoInstanceId(-1)` の invalid sentinel。
- `SelfhostHirChildRange(-1, 0)`、`SelfhostHirParamRange(-1, 0)` の empty range sentinel。
- HIR expression payload が kind ごとに所有 field を分離できていないこと。
