---
id: ISS-20260428T212439976Z-SELF-HOST-TYPE-KIND-LACKS-RUST-PRIMI-43D4589C
title: "self-host type kind lacks Rust primitive parity"
area: selfhost
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/ty/ty.nepl
---

# ISS-20260428T212439976Z-SELF-HOST-TYPE-KIND-LACKS-RUST-PRIMI-43D4589C: self-host type kind lacks Rust primitive parity

## 概要

SelfhostTypeKind currently covers Unit/Bool/I32/I64/U8/Char/Str/Function, but the Rust type context and parser expose F32 and Never as first-class primitive kinds and handle i64/f64 as named numeric types. The self-host type layer therefore cannot model the full primitive surface used by current NEPLg2 signatures.

## 対象

- `stdlib/neplg2/core/ty/ty.nepl`

## 根拠

- 未記入

## 問題

SelfhostTypeKind currently covers Unit/Bool/I32/I64/U8/Char/Str/Function, but the Rust type context and parser expose F32 and Never as first-class primitive kinds and handle i64/f64 as named numeric types. The self-host type layer therefore cannot model the full primitive surface used by current NEPLg2 signatures.

## 影響

A self-host checker built on the current type arena would either reject valid f32/never signatures, encode them as ad hoc named types, or diverge from Rust diagnostics and overload behavior.

## 修正方針

Extend the self-host type model with explicit Rust-parity primitive coverage, define canonical/source spellings for unit/never/floating and named numeric aliases, and add parity doctests against representative signatures.

## 検証

Run ty/prelude focused doctests and self-host signature parity fixtures once the type model is extended.
