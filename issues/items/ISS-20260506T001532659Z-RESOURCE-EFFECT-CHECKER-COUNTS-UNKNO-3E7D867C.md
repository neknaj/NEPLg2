---
id: ISS-20260506T001532659Z-RESOURCE-EFFECT-CHECKER-COUNTS-UNKNO-3E7D867C
title: "Resource effect checker counts unknown effects without diagnostic"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/compiler.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T001532659Z-RESOURCE-EFFECT-CHECKER-COUNTS-UNKNO-3E7D867C: Resource effect checker counts unknown effects without diagnostic

## 概要

ResourceEffectBoundaryEngine handles EffectOp::Unknown by incrementing unknown_ops only. Although normal indirect calls now lower to EffectOp::IndirectCall, any remaining or future unknown effect in Resource IR can pass the compiler effect boundary without a diagnostic.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/compiler.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `nepl-core/src/resource/effect_check.rs` は `EffectOp::Unknown` を `unknown_ops` に count するだけで、`ResourceEffectBoundaryDiagnostic` を生成していなかった。
- `EffectOp::IndirectCall { effect }` の導入により通常の indirect call lowering は typed effect を保持するようになったが、`EffectOp::Unknown` variant 自体は Resource IR に残っている。
- `doc/neplg2/static_check_soundness_review_20260430.md` でも、final design では `EffectOp::Unknown` を通常の lowering 成功状態として扱わない方針になっている。

## 問題

ResourceEffectBoundaryEngine handles EffectOp::Unknown by incrementing unknown_ops only. Although normal indirect calls now lower to EffectOp::IndirectCall, any remaining or future unknown effect in Resource IR can pass the compiler effect boundary without a diagnostic.

## 影響

Resource IR cannot be the final static-check authority if unknown effects are non-errors. A lowering regression or future resource operation could silently bypass effect safety and leave old typecheck as the only guard.

## 修正方針

Treat EffectOp::Unknown as Resource IR lowering incompleteness, emit a typed ResourceEffectBoundaryDiagnostic, and map it to resource.lower.incomplete at the compiler gate. Keep operation counts for reporting, but do not allow count-only unknown effects.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_effect_check_reports_unknown_effect_as_lowering_incomplete -- --nocapture; cargo test -p nepl-core compiler::tests::resource_effect_gate_maps_unknown_effect_to_lower_incomplete_code --lib; node nodesrc/issues.js check; git diff --check

## 対応結果

`EffectOp::Unknown` を count-only の報告値ではなく、Resource IR lowering incompleteness として扱うようにした。

- `ResourceEffectBoundaryDiagnostic::UnknownEffect` を追加し、unknown effect の function / reason / span を保持する。
- `ResourceEffectBoundaryEngine` は `unknown_ops` count を維持しつつ、unknown effect を必ず diagnostic にする。
- compiler gate は `UnknownEffect` を `resource.lower.incomplete` へ写像する。
- 手作業 Resource IR test で既知 pure callback を `EffectOp::Unknown` としていた箇所は `EffectOp::IndirectCall { effect: Effect::Pure }` へ修正し、unknown effect を正常系として使わないようにした。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_reports_unknown_effect_as_lowering_incomplete -- --nocapture`: passed
- `cargo test -p nepl-core compiler::tests::resource_effect_gate_maps_unknown_effect_to_lower_incomplete_code --lib`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_ -- --nocapture`: passed
