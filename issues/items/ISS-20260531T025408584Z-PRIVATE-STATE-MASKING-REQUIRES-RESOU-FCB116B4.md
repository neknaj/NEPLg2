---
id: ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4
title: "Private state masking requires Resource IR escape proof"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/effect_identity.rs; nepl-core/src/resource/model.rs"
---

# ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4: Private state masking requires Resource IR escape proof

## 概要

PrivateCache and PrivateState can be folded to Pure only when their region, storage identity, raw pointer provenance, references, and observation APIs do not escape the trusted boundary.

## 対象

- `nepl-core/src/resource/effect_check.rs; nepl-core/src/resource/effect_identity.rs; nepl-core/src/resource/model.rs`

## 根拠

- `PrivateCache` / `PrivateState` は内部 mutation を持つため、それ自体を `Pure` として扱ってはいけない。
- `Pure` へ mask できるのは、fresh private region が return value、global state、public field、raw pointer、reference、owner token、function identity へ escape しない場合だけである。
- 現行 Resource IR effect checker は raw memory identity escape、raw pointer alias、function alias を追跡する基礎を持つが、private state region を一つの proof domain として扱う設計はまだない。
- `memo_call` の安全性は `func` が pure であることだけでなく、cache hit/miss/size/clear/reference が public API へ出ないことにも依存する。

## 問題

PrivateCache and PrivateState can be folded to Pure only when their region, storage identity, raw pointer provenance, references, and observation APIs do not escape the trusted boundary.

## 影響

If private state effects are masked without a dedicated Resource IR escape proof, memo_call or later private buffers can leak cache storage identity or public state while still passing as Pure.

## 修正方針

Add a Resource IR proof domain for fresh private regions, derived pointer/reference/owner-token escape, public observation APIs, and allowed private operations before general PrivateState masking is accepted.

## 2026-05-31 design checkpoint

- [NEPLg2 private effect / memoization purity design](../../doc/neplg2/private_effect_memoization_purity_design.md) に、Resource IR が mask 前に証明する最低条件を列挙した。
- Phase 1 の `memo_call` では、SourceCapability で trusted stdlib memo implementation だけに private cache boundary use-site を与える。
- `PrivateCache rho` は `rho` が fresh / non-escaping と証明されるまで `Pure` へ fold しない。
- raw memory operation は直接 `Pure` へ戻さず、trusted private capability と provenance によって `PrivateCache rho` へ分類できる場合だけ mask 対象にする。

## 検証

Focused Resource IR tests that accept non-escaping private cache operations and reject returning cache, cache references, raw pointers, stats, clear handles, or passing private cache to impure/unknown calls.
