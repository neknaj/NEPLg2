---
id: ISS-20260604T033643672Z-DIAGS-BY-VALUE-OBSERVER-MUST-CLOSE-O-C6D3EAEA
title: "Diags by-value observer must close owner after borrowed observation"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/alloc/diag/error/diags.nepl
---

# ISS-20260604T033643672Z-DIAGS-BY-VALUE-OBSERVER-MUST-CLOSE-O-C6D3EAEA: Diags by-value observer must close owner after borrowed observation

## 概要

node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js reports that by-value diags_has_errors must close the Diags owner after observing via &Diags. The current overload set exposes a by-value observer but the source policy cannot prove it frees the owner after borrowed observation. This violates the Zenn policy around ownership, pure observation, and static memory safety.

## 対象

- `stdlib/alloc/diag/error/diags.nepl`

## 根拠

- 未記入

## 問題

node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js reports that by-value diags_has_errors must close the Diags owner after observing via &Diags. The current overload set exposes a by-value observer but the source policy cannot prove it frees the owner after borrowed observation. This violates the Zenn policy around ownership, pure observation, and static memory safety.

## 影響

Diagnostic containers can become a precedent for observer APIs that consume owners without an explicit cleanup contract, weakening ownership conventions across selfhost diagnostics.

## 修正方針

Implement the by-value overload as borrow-then-free-then-return, document the ownership contract in the doc comment, and add focused doctests plus future regular tests for empty, warning-only, error-containing, and consumed-owner cases.

## 検証

Run node nodesrc/test_stdlib_diag_error_no_unsafe_unwraps.js and focused diag doctests.
