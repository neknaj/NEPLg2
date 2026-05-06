---
id: ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232
title: "Resource authority deep prefix regressions exceed local test budget"
area: core
status: fixed
resolved: true
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

## 解決内容

根本原因は 2 つあった。

- `RawCellAddressAliases::copy_alias_or_seed` が、通常の i32 value copy まで raw-address alias group として新規 seed していた。1105 段 prefix call では普通の scalar copy が巨大な alias group になり、initialized / effect / owner gate がその group を何度も走査していた。
- transparent raw-address return lowering と raw alias return summary の適用が広すぎ、`fn inc(x): x` のような通常の i32 identity まで raw address helper として扱っていた。これにより、Resource IR authority path が普通の pure prefix chain に raw-address alias operation を大量に materialize していた。

修正では、通常の value copy と明示 raw-address relation を分離した。

- `copy_alias_if_tracked` は既に追跡中の raw-address alias と scalar fact だけを伝播し、通常 i32 copy では新規 raw alias group を作らない。
- `copy_explicit_raw_address_alias` を追加し、`RawAddressAlias` / `RawAddressView` のような明示 raw-address operation だけが raw alias group を seed する。
- raw memory address と `MemPtr.raw` の同一性を失わないよう、stable local origin を value-origin fact として別管理し、raw memory address canonicalization 時だけ参照する。これは alias group ではないため、普通の scalar chain を巨大 group にしない。
- transparent raw-address return lowering は、bare i32 parameter return を raw helper とみなさない。`add` / `sub` / `mem_ptr_*` / `region_*` など raw-address operation の operand として現れた parameter だけを raw address projection として扱う。
- function raw alias return summary の適用は、未追跡 scalar を新規 raw alias として seed せず、既に raw として追跡中の relation だけを伝播する。

この変更により、deep-prefix regression は 240 秒 timeout / 359 秒級から 10 秒未満に戻り、同時に higher-order function value raw write の memory-safety regression も維持した。

## 検証結果

- `cargo fmt --check`: passed
- `cargo test -p nepl-core --test check_pipeline resource_static_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed, finished in 9.33s
- `cargo test -p nepl-core --test check_pipeline prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed, finished in 9.39s
- `cargo test -p nepl-core --test check_pipeline resource_drop_insertion_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed, finished in 3.09s
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_rejects -- --nocapture`: 8 passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_gate_order.js`: passed
- `node nodesrc/issues.js check`: passed
