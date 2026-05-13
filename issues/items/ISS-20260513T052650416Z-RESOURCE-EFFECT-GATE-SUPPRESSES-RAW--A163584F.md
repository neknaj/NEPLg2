---
id: ISS-20260513T052650416Z-RESOURCE-EFFECT-GATE-SUPPRESSES-RAW--A163584F
title: "Resource effect gate suppresses raw identity escape in raw memory boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/compiler.rs
---

# ISS-20260513T052650416Z-RESOURCE-EFFECT-GATE-SUPPRESSES-RAW--A163584F: Resource effect gate suppresses raw identity escape in raw memory boundary

## 概要

resource_effect_boundary_diagnostic_is_raw_boundary_allowed suppresses RawAddressEscapeFromInternalAlloc when the diagnostic span belongs to a raw-memory-boundary source. Raw identity escape is itself the Resource IR proof that an InternalAlloc identity leaked, so a source capability must not erase it.

## 対象

- `nepl-core/src/compiler.rs`

## 根拠

- `resource_effect_boundary_diagnostic_is_raw_boundary_allowed` は `UnsafeMemoryInPureFunction` と `RawAddressEscapeFromInternalAlloc` を同じ分岐に入れていた。
- その分岐は diagnostic span の file が `raw_memory_boundary_allowed` なら `true` を返し、compiler gate が診断を捨てていた。
- `RawAddressEscapeFromInternalAlloc` は Resource IR が internal allocation identity の escape を検出した結果であり、raw-memory-boundary capability で消すと「証明された違反」を例外表で無効化することになる。

## 問題

resource_effect_boundary_diagnostic_is_raw_boundary_allowed suppresses RawAddressEscapeFromInternalAlloc when the diagnostic span belongs to a raw-memory-boundary source. Raw identity escape is itself the Resource IR proof that an InternalAlloc identity leaked, so a source capability must not erase it.

## 影響

A raw-memory-boundary implementation can return or store an allocation identity through a pure surface without compiler error, making effect safety depend on path-granted privilege instead of Resource IR escape proof.

## 修正方針

Keep transitional raw-memory-boundary handling for unsafe raw memory operations separate, but never suppress RawAddressEscapeFromInternalAlloc. Add a focused unit test so raw identity escape remains an unconditional Resource IR diagnostic.

## 検証

Run focused compiler effect-gate tests, move_effect doctests, source policy, issue check, and formatting checks.

## 2026-05-13 修正

`resource_effect_boundary_diagnostic_is_raw_boundary_allowed` を分離し、`RawAddressEscapeFromInternalAlloc` は常に `false` を返すようにした。これにより raw-memory-boundary source であっても、Resource IR が raw identity escape を証明した場合は `resource.raw.identity_escape` diagnostic が gate で維持される。

`compiler.rs` の unit test に `resource_effect_gate_never_suppresses_raw_identity_escape_in_raw_boundary` を追加し、raw boundary capability を持つ `SourceMap` 上でも raw identity escape が抑制されないことを固定した。source policy も更新し、`RawAddressEscapeFromInternalAlloc` が unsafe-memory raw-boundary suppression と同じ分岐へ戻らないよう監視する。

検証:

- `cargo test -p nepl-core compiler::tests::resource_effect_gate_ -- --nocapture`: 5/5 pass。
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass。
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-effect-boundary-move-effect.json -j 1 --dist web/dist`: 113/113 pass。
