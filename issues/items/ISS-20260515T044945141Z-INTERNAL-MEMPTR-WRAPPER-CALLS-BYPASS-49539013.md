---
id: ISS-20260515T044945141Z-INTERNAL-MEMPTR-WRAPPER-CALLS-BYPASS-49539013
title: "internal MemPtr wrapper calls bypass raw memory caller boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/effect*.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260515T044945141Z-INTERNAL-MEMPTR-WRAPPER-CALLS-BYPASS-49539013: internal MemPtr wrapper calls bypass raw memory caller boundary

## 概要

User source could directly import `core/mem/internal`, call `mem_ptr_wrap` on a fixed positive raw address, and then pass the forged `MemPtr` to checked `store_i32`. The raw operation happened inside compiler-owned stdlib source, so the call site compiled even though the caller had not proven raw-memory-boundary authority.

## 対象

- `nepl-core/src/resource/effect_checked_memptr.rs`
- `nepl-core/src/resource/effect_check.rs`
- `nepl-core/src/resource/effect_summary_pointer.rs`
- `nepl-core/src/resource/effect_summary_identity.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/memory_safety.n.md`

## 根拠

- 静的検査大規模修正 Stage 5/6 の方針では、checked stdlib wrapper を名前で信頼するのではなく、Resource IR 上で raw address / `MemPtr.raw` の provenance を証明する必要がある。
- `mem_ptr_wrap 16` のような user source 由来の正の raw address は allocator / region owner 由来でも null sentinel でもないため、checked wrapper に渡しても memory safety を証明できない。
- 一方で `mem_ptr_wrap 0` は checked wrapper 内の null guard へ進む sentinel であり、positive forged address と区別して受理する必要がある。

## 問題

User source could directly import `core/mem/internal`, call `mem_ptr_wrap` on a fixed positive raw address, and then pass the forged `MemPtr` to checked `store_i32`. The raw operation happened inside compiler-owned stdlib source, so the call site compiled even though the caller had not proven raw-memory-boundary authority.

## 影響

Safe source could construct arbitrary non-owning `MemPtr` values and use checked pointer wrappers as raw memory write/read entry points. This weakened Stage 6 `MemPtr = non-owning view` discipline and made public/internal module separation depend on convention rather than compiler proof.

## 修正方針

Fixed by adding a typed `ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary` proof gate for checked `MemPtr` raw-memory wrapper calls.

- Checked `load` / `store` / `fill` / bulk operations now inspect `MemPtr.raw` arguments in Resource IR.
- A checked call is accepted only when the raw field is proven to come from allocator / region allocation identity, or when it aliases an `i32` null sentinel (`<= 0`) that the checked wrapper guards before raw memory access.
- The proof is source-derived: raw identity is propagated through allocator and region returns, `Result` payload match binds, aggregate fields, borrows, raw address views, direct calls, indirect callback returns, and local reads. It does not special-case a stdlib allowlist.
- Pointer return summaries were redesigned from whole-return alias booleans to projection-aware parameter/return mappings, so `mem_ptr_wrap` can prove `return.field0` aliases the literal argument without trusting the function name.
- `RawAddressView` now considers projection prefixes as provenance candidates, preserving owner aggregate identity across reference, field, and offset projections.

## 検証

- Rust integration:
  - forged positive `MemPtr` passed to checked store is rejected with `resource.raw.memory_outside_boundary`.
  - null sentinel `MemPtr` reaches the checked load guard.
  - allocator, region, `region_ptr_at`, and callback-returned region pointers preserve checked wrapper provenance.
- Doctest:
  - `tests/stdlib/memory_safety.n.md` includes forged positive `MemPtr` compile-fail regression.
  - full focused run passes: 37/37.
- Focused verification:
  - `cargo check -p nepl-core`
  - `cargo test -p nepl-core --test resource_ir compile_ -- --nocapture`
  - `cargo test -p nepl-core --lib resource_effect_gate -- --nocapture`
  - `node nodesrc/test_resource_checker_responsibility.js`
  - `trunk build`
  - `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-internal-memptr-call-boundary-after.json -j 1 --dist web/dist --assert-io`
