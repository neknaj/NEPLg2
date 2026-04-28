---
id: ISS-20260428T225903679Z-SELF-HOST-MONO-STAGE-ONLY-EXPOSES-MA-FC3889CC
title: "self-host mono stage only exposes marker API and lacks instance key model"
area: selfhost
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md"
---

# ISS-20260428T225903679Z-SELF-HOST-MONO-STAGE-ONLY-EXPOSES-MA-FC3889CC: self-host mono stage only exposes marker API and lacks instance key model

## 概要

stdlib/neplg2/core/mono/mono.nepl is still a Stage 0 marker, so later monomorphize work has no typed representation for a generic function instance key or deterministic symbol identity.

## 対象

- `stdlib/neplg2/core/mono/mono.nepl, tests/stdlib/neplg2_mono.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S4 は `mono/` で instance cache と name mangling を分離するとしている。
- `stdlib/neplg2/core/mono/mono.nepl` は現在 `selfhost_mono_stage0` だけを返す 26 行の marker module で、generic instance を識別する typed key を持たない。
- `doc/neplg2/self_host_execution_plan.md` でも S4 commit 単位として `selfhost/s4-mono-instance` が予定されている。

## 問題

stdlib/neplg2/core/mono/mono.nepl is still a Stage 0 marker, so later monomorphize work has no typed representation for a generic function instance key or deterministic symbol identity.

## 影響

S4 monomorphize work would otherwise spread ad hoc module/function/type-argument tuples and mangling seeds across lowering, cache, and codegen, making parity with the Rust compiler harder to test.

## 修正方針

Add a small Copy instance key model with module/function/type-argument range fields, equality, validity checks, and a deterministic mangle seed helper. Keep cache storage and trait impl lookup for later issues.

## 検証

Run mono focused doctests, self-host focused tests that depend on mono, node nodesrc/issues.js check, and git diff --check.
