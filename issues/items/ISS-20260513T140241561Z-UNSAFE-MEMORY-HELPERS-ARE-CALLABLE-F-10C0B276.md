---
id: ISS-20260513T140241561Z-UNSAFE-MEMORY-HELPERS-ARE-CALLABLE-F-10C0B276
title: "unsafe memory helpers are callable from impure user source outside raw boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/effect_check.rs; nepl-core/src/compiler.rs; tests/stdlib/memory_safety.n.md"
---

# ISS-20260513T140241561Z-UNSAFE-MEMORY-HELPERS-ARE-CALLABLE-F-10C0B276: unsafe memory helpers are callable from impure user source outside raw boundary

## 概要

User source could import core/mem/raw and call raw unsafe memory helpers from an impure function, because Resource IR only reported UnsafeMemoryInPureFunction and treated impure functions as allowed.

## 対象

- `nepl-core/src/resource/effect_check.rs; nepl-core/src/compiler.rs; tests/stdlib/memory_safety.n.md`

## 根拠

- A direct user-source probe using `#import "core/mem/raw" as *` and `fn main <()*>i32>` with `store_i32 16 7` / `load_i32 16` compiled before this fix.
- The Resource IR effect boundary check reported `UnsafeMemoryInPureFunction` only for pure callers, so impure user code bypassed the raw-memory-boundary capability.
- During implementation, existing doctests revealed a second root cause: call effects use raw helper names for both direct raw address helpers and safe `MemPtr` wrappers. Therefore the new raw-boundary diagnostic must be based on `ResourceOp::RawMemory`, not on every `EffectOp::UnsafeMemory`.

## 問題

User source could import core/mem/raw and call raw unsafe memory helpers from an impure function, because direct raw memory operations did not have a raw-boundary diagnostic independent from pure/impure effect checking.

## 影響

Raw memory discipline can be bypassed by writing impure user code, so unsafe memory operations are not limited to compiler-owned raw-memory-boundary source.

## 修正方針

Added `ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary` and `ResourceRawDiagnosticCode::MemoryOutsideBoundary`. The effect checker now emits this diagnostic from actual `ResourceOp::RawMemory` operations, so safe `MemPtr` wrapper calls still participate in pure-effect diagnostics without being mistaken for direct raw-boundary violations. The compiler gate suppresses the diagnostic only when the operation span belongs to a source file with `raw_memory_boundary` capability.

Doctests that directly used raw address helpers from user source were updated to the new policy: safe API tests use `MemPtr` wrappers, and remaining direct raw usage is a `compile_fail` regression with `resource.raw.memory_outside_boundary`.

## 検証

- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core resource_effect_gate -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_does_not_treat_mem_ptr_store_wrapper_as_direct_raw_memory -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-raw-boundary-memory-safety.json -j 1 --dist web/dist`: total=30, passed=30
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-raw-boundary-move-effect.json -j 4 --dist web/dist`: total=113, passed=113
- `node nodesrc/test_resource_gate_order.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
