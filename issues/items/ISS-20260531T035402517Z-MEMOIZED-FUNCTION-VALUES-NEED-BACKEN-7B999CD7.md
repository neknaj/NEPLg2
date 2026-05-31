---
id: ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7
title: "memoized function values need backend representation and identity-observation ban"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs"
---

# ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7: memoized function values need backend representation and identity-observation ban

## 概要

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 対象

- `nepl-core/src/codegen; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/effect_check.rs`

## 根拠

- 未記入

## 問題

Existing backend function values are lowered as table indices or i32-like ids and do not carry private cache environment state, while memo_call returns a function value with hidden private cache storage.

## 影響

Without a backend representation and identity-observation ban, memoized function values can either be impossible to lower or can leak closure/cache allocation identity through equality, hash, raw store/load, cast, layout query, or debug output.

## 修正方針

Choose a Phase 1 representation for memoized functions, such as compiler-generated wrappers with hidden private cache regions or a closure object with sealed identity, and forbid pure public APIs that observe function address, closure allocation id, cache region id, equality, hash, or raw representation.

## 検証

Regression tests should accept calling a memoized pure named function and reject identity/hash/address/cast/raw-store observation, function-value key usage, public cache field exposure, and backend paths that require an unsealed closure id.
