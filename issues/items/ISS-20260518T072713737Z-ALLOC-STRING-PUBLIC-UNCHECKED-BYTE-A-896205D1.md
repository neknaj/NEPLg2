---
id: ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1
title: "alloc string public unchecked byte access needs safe boundary redesign"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/access.nepl, stdlib/**, tests/stdlib/memory_safety.n.md"
---

# ISS-20260518T072713737Z-ALLOC-STRING-PUBLIC-UNCHECKED-BYTE-A-896205D1: alloc string public unchecked byte access needs safe boundary redesign

## 概要

alloc/string/access exposes string_byte_at_unchecked through the root alloc/string facade. Many stdlib and selfhost modules call it after local bounds reasoning, but ordinary callers can also direct-call it with arbitrary indices because the proof obligation is only documented, not represented in the type system or Resource IR API boundary.

## 対象

- `stdlib/alloc/string/access.nepl, stdlib/**, tests/stdlib/memory_safety.n.md`

## 根拠

- 未記入

## 問題

alloc/string/access exposes string_byte_at_unchecked through the root alloc/string facade. Many stdlib and selfhost modules call it after local bounds reasoning, but ordinary callers can also direct-call it with arbitrary indices because the proof obligation is only documented, not represented in the type system or Resource IR API boundary.

## 影響

Raw string-layout reads remain available as a public function. This is safer than raw address exposure because it returns a byte value, but it still leaves a memory-safety precondition outside compiler-enforced proof and conflicts with the Stage 6 goal that public APIs do not rely on caller discipline for raw storage access.

## 修正方針

Redesign string byte access so unchecked raw reads are either private to compiler-owned modules or require an explicit proof/witness produced by a checked bounds operation. Migrate stdlib call sites to checked helpers, scanner/range APIs, or a typed bounded-index API rather than exposing the raw unchecked reader through the root facade.

## 検証

Add compile_fail coverage for ordinary direct calls after migration, source policy preventing root re-export of unchecked raw byte readers, and focused string/parser/scanner doctests.
