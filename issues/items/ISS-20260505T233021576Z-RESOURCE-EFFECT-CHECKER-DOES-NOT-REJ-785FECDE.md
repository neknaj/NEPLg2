---
id: ISS-20260505T233021576Z-RESOURCE-EFFECT-CHECKER-DOES-NOT-REJ-785FECDE
title: "Resource effect checker does not reject direct host effects in pure functions"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T233021576Z-RESOURCE-EFFECT-CHECKER-DOES-NOT-REJ-785FECDE: Resource effect checker does not reject direct host effects in pure functions

## 概要

ResourceEffectBoundaryEngine counts EffectOp::ExternalIo and EffectOp::Nondet but does not route them through the pure function impure-effect diagnostic path. A Resource IR function with a direct host effect can remain diagnostic-free even though host I/O and nondeterminism surface-fold to Impure.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `InternalEffect::{ExternalIo,Nondet}` は surface fold で `Impure` へ分類されるが、Resource IR の `EffectOp::{ExternalIo,Nondet}` は `check_effect` で count されるだけだった。
- `ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction` は user direct call / indirect call だけを表せる `ResourceEffectCallKind` を持ち、host effect を typed に診断する variant がなかった。
- そのため、Resource IR 上で direct host effect を含む pure function を構成すると、effect boundary diagnostic が出ない経路が残っていた。

## 問題

ResourceEffectBoundaryEngine counts EffectOp::ExternalIo and EffectOp::Nondet but does not route them through the pure function impure-effect diagnostic path. A Resource IR function with a direct host effect can remain diagnostic-free even though host I/O and nondeterminism surface-fold to Impure.

## 影響

Resource IR cannot be authoritative for effect safety while direct host effects bypass pure-boundary diagnostics. Later lowering or raw-body migration could hide I/O or nondeterminism behind a pure Resource function.

## 修正方針

Add typed ResourceEffectCallKind variants for ExternalIo and Nondet, route these effects through check_call_effect with Effect::Impure, and add focused Resource IR regressions for pure-host-effect diagnostics.

## 対応

- `ResourceEffectCallKind::{ExternalIo,Nondet}` を追加し、diagnostic の call 種別でも host operation identity を typed enum として保持するようにした。
- `EffectOp::{ExternalIo,Nondet}` は count 記録後、`Effect::Impure` として `check_call_effect` へ通すようにした。
- compiler diagnostic 表示も typed call kind の exhaustive match に更新した。
- Resource IR regression で pure function 内の direct `fd_write` と `random_get` が `ImpureCallInPureFunction` になることを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_rejects_direct_host_effects_in_pure_function -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_ -- --nocapture`: 22 passed
- `node nodesrc/issues.js check`: commit 前に実行
