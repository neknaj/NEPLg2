---
id: ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232
title: "Resource authority deep prefix regressions exceed local test budget"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/tests/check_pipeline.rs, nepl-core/src/compiler.rs, nepl-core/src/resource"
---

# ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232: Resource authority deep prefix regressions exceed local test budget

## 概要

During the RV-CORE-009 closure audit, focused deep-prefix check_pipeline regressions for resource_static_check_accepts_deep_prefix_chain_without_stack_overflow and prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow exceeded a 240s local command budget. The previous stack-overflow issue was fixed, but the current Resource IR authority path may still have excessive compile-time complexity on long prefix-call chains.

## 対象

- `nepl-core/tests/check_pipeline.rs, nepl-core/src/compiler.rs, nepl-core/src/resource`

## 根拠

- `node nodesrc/test_resource_gate_order.js` は即時に `resource gate authority ok` で通過し、pipeline authority の source policy は成立している。
- 一方、同じ監査で `cargo test -p nepl-core --test check_pipeline resource_static_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture` と `cargo test -p nepl-core --test check_pipeline prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture` はそれぞれ 240 秒で local command timeout になった。
- 旧 stack overflow 回避 regression は残すべきだが、現状の 1105-call chain が Resource IR authority path の O(n^2) 以上の挙動を露出しているのか、debug build で本質的に重いだけなのかは切り分けられていない。

## 問題

During the RV-CORE-009 closure audit, focused deep-prefix check_pipeline regressions for resource_static_check_accepts_deep_prefix_chain_without_stack_overflow and prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow exceeded a 240s local command budget. The previous stack-overflow issue was fixed, but the current Resource IR authority path may still have excessive compile-time complexity on long prefix-call chains.

## 影響

If the deep-prefix regressions require several minutes locally, CI can lose signal or timeout while trying to prove that the Resource IR authority path does not regress. This may hide whether the problem is algorithmic complexity in monomorphize/resource lowering/summary gates or an inherently heavy regression size.

## 修正方針

Profile the 1105-call check_pipeline cases stage by stage, measure monomorphize, Resource IR lowering, cell/borrow/effect/owner gates, and drop elaboration separately, then either optimize the hot path or resize/split the regression only if profiling proves the current workload is intentionally too large.

## 検証

Run the exact check_pipeline deep-prefix tests with stage timing and confirm they complete within the intended local/CI budget without weakening Resource IR gates.
