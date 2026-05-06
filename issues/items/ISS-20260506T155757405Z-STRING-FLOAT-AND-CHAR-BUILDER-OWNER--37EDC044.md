---
id: ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044
title: "String float and char builder owner chains fail strict ResourceIR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/alloc/string/float.nepl, stdlib/alloc/string/builder.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md"
---

# ISS-20260506T155757405Z-STRING-FLOAT-AND-CHAR-BUILDER-OWNER--37EDC044: String float and char builder owner chains fail strict ResourceIR

## 概要

Focused string verification after alloc/string facade split reaches ResourceIR and reports resource.owner.use_after_move/reserved in from_f64_append_fraction_result, from_f64_build_fixed_result, string_char.n.md char slice checks, and ByteBuilder finish chains. The failures occur on Result-returning builder owners that should transfer exactly once through Ok arms.

## 対象

- `stdlib/alloc/string/float.nepl, stdlib/alloc/string/builder.nepl, stdlib/alloc/io.nepl, tests/stdlib/string_char.n.md`

## 根拠

- 未記入

## 問題

Focused string verification after alloc/string facade split reaches ResourceIR and reports resource.owner.use_after_move/reserved in from_f64_append_fraction_result, from_f64_build_fixed_result, string_char.n.md char slice checks, and ByteBuilder finish chains. The failures occur on Result-returning builder owners that should transfer exactly once through Ok arms.

## 影響

String numeric formatting and char/byte builder tests cannot be used as a clean regression signal under mandatory memory-safety checking. This can hide real builder leaks or push developers toward weakening ResourceIR diagnostics.

## 修正方針

Trace the builder owner summaries and call-site Result arm refinement without weakening ResourceIR. If stdlib code is relying on ambiguous owner flow, rewrite the builder chains so each owner is consumed or freed in a statically visible path and add focused regression tests for from_f64 and char builders.

## 検証

Run focused string float and string_char doctests, source policy string owner checks, and ResourceIR owner regressions.
