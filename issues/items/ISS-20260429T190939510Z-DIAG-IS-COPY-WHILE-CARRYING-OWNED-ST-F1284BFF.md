---
id: ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF
title: "Diag is Copy while carrying owned string fields and lacks a consumption contract"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md, stdlib/alloc/collections/**"
---

# ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF: Diag is Copy while carrying owned string fields and lacks a consumption contract

## 概要

Strict Resource IR now reports a leak when a collection test matches Result::Err d and inspects diag_std_error_kind_str d: the local Diag keeps an owned message field alive. Diag currently implements Copy even though it contains str fields such as message, notes, help, and optional source. That makes diagnostic values look lightweight while still carrying ownership obligations in failure branches.

## 対象

- `stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md, stdlib/alloc/collections/**`

## 根拠

- 未記入

## 問題

Strict Resource IR now reports a leak when a collection test matches Result::Err d and inspects diag_std_error_kind_str d: the local Diag keeps an owned message field alive. Diag currently implements Copy even though it contains str fields such as message, notes, help, and optional source. That makes diagnostic values look lightweight while still carrying ownership obligations in failure branches.

## 影響

Collection and self-host tests cannot safely inspect rich Diag values returned from Err without either leaking strings or relying on Copy semantics that conflict with the owner model. Self-host diagnostics need a clear type-safe contract before diagnostic aggregation grows larger.

## 修正方針

Redesign Diag ownership: either make the low-level collection error path return a Copy-only StdErrorKind/lightweight code, or make Diag a non-Copy owned diagnostic with explicit borrowed accessors and a free/drop contract. Add fixtures that match Err(Diag), inspect the kind, and close the diagnostic owner without weakening Resource IR.

## 検証

Run node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1 --dist web/dist and the diag/error stdlib suites after the ownership contract is redesigned.
