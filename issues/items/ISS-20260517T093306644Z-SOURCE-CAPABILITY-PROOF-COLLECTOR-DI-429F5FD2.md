---
id: ISS-20260517T093306644Z-SOURCE-CAPABILITY-PROOF-COLLECTOR-DI-429F5FD2
title: "Source capability proof collector dispatch remains domain-specific"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T093306644Z-SOURCE-CAPABILITY-PROOF-COLLECTOR-DI-429F5FD2: Source capability proof collector dispatch remains domain-specific

## 概要

Source capability proof collection still calls raw memory, owner aggregate, and compiler-memory-field evidence collectors through hand-written per-domain methods. Adding a new proof domain requires editing multiple observer arms manually, so the compiler does not make it easy to see whether all syntactic evidence events are covered.

## 対象

- `nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- source capability proof は `SourceCapabilityUseSite` で exact use-site proof 化されているが、proof collector 側に domain 別 dispatch が残ると、追加した proof domain が call-head / intrinsic / raw-body などへ適用されたかを型構造で確認しにくい。

## 問題

Source capability proof collection still calls raw memory, owner aggregate, and compiler-memory-field evidence collectors through hand-written per-domain methods. Adding a new proof domain requires editing multiple observer arms manually, so the compiler does not make it easy to see whether all syntactic evidence events are covered.

## 影響

This leaves static-check authority dependent on collector wiring discipline instead of a generic proof rule engine. Future Resource IR/static-check capability domains can be forgotten for call-head, alias, intrinsic, or raw-body events, weakening the exact use-site proof model required by Stage 6.

## 修正方針

Introduce a typed source capability proof rule dispatcher. Each observer event should be represented as a typed event enum and routed through one dispatcher that applies all proof rules via exhaustive matches. Domain-specific evidence can remain in modules, but event coverage must be centralized and enforced by enum/match structure rather than ad hoc collector calls.

## 検証

Add policy checks that proof.rs uses SourceCapabilityProofEvent and dispatch_source_capability_proof_event, and that observe_call_head_symbol / observe_fn_alias_target / observe_intrinsic route through the dispatcher. Run focused nepl-core source capability tests, static-check responsibility policy, issue validation, and diff checks.

## 修正内容

`SourceCapabilityProofEvent` と `dispatch_source_capability_proof_event` を追加し、source capability observer は typed event を dispatcher へ渡すだけにした。raw memory / owner aggregate / compiler memory field の domain-specific evidence 判定は各 module に残しつつ、どの構文イベントにどの proof rule を適用するかは `rule.rs` の exhaustive `match event` に集約した。

`SourceCapabilityProofSink` は collector state への最小 interface とし、raw helper function frame / top-level raw call propagation は既存の証明状態を維持する。これにより、新しい proof event を追加した場合は `SourceCapabilityProofEvent` と dispatcher match の更新が Rust の網羅性検査で必要になる。

## 完了確認

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core raw_memory_boundary --lib -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundary --lib -- --nocapture`
- `cargo test -p nepl-core compiler_memory_field_boundary --lib -- --nocapture`
- `cargo test -p nepl-core source_map::tests --lib -- --nocapture`
