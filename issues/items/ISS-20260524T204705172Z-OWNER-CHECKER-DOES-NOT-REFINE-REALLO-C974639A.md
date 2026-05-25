---
id: ISS-20260524T204705172Z-OWNER-CHECKER-DOES-NOT-REFINE-REALLO-C974639A
title: "Owner checker does not refine realloc failure flags through loop guards"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/resource/owner_control.rs; nepl-core/src/resource/owner_state.rs"
---

# ISS-20260524T204705172Z-OWNER-CHECKER-DOES-NOT-REFINE-REALLO-C974639A: Owner checker does not refine realloc failure flags through loop guards

## 概要

Raw realloc failure can leave an owner as MaybeFreed after path merge, and a later guard such as done == false does not recover the success-only owner state. Source fixtures that call fd_read after a done flag guard therefore fail ExternalIoPayloadExtent even when the guarded path has a live buffer.

## 対象

- `nepl-core/src/resource/owner_control.rs; nepl-core/src/resource/owner_state.rs`

## 根拠

- 未記入

## 問題

Raw realloc failure can leave an owner as MaybeFreed after path merge, and a later guard such as done == false does not recover the success-only owner state. Source fixtures that call fd_read after a done flag guard therefore fail ExternalIoPayloadExtent even when the guarded path has a live buffer.

## 影響

Grow-loop style raw buffer code must keep host I/O inside the direct realloc success/no-realloc branches. More natural flag-driven loops remain harder to verify, which can affect future stdlib/selfhost raw-boundary implementations.

## 修正方針

Teach Resource IR owner checking to carry boolean path facts that relate loop flags to raw realloc success/failure, or introduce a structured realloc-result proof that keeps success-only owner state available after equivalent guards.

## 検証

Add a Resource IR fixture where a raw buffer is reallocated, failure sets a done flag, and a later done == false guard permits fd_read on buf + len without reporting ExternalIoPayloadExtent NoFreeObligation.
