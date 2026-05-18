---
id: ISS-20260518T060850538Z-CHECKED-MEMPTR-PROVENANCE-REJECTS-RE-CA3CAB00
title: "checked MemPtr provenance rejects RegionToken-derived pointer on main"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/effect_return_summary_filter_tests.rs, nepl-core/tests/resource_ir.rs, stdlib/core/mem/pointer"
---

# ISS-20260518T060850538Z-CHECKED-MEMPTR-PROVENANCE-REJECTS-RE-CA3CAB00: checked MemPtr provenance rejects RegionToken-derived pointer on main

## 概要

Current main suppresses raw identity return summaries for direct RegionToken owner tokens. alloc_region returns Result<RegionToken<T>, str>, so RegionToken.raw allocation identity is not propagated to region_ptr, and checked MemPtr store/load/fill reports resource.raw.memory_outside_boundary even though the pointer is derived from a compiler-issued RegionToken.

## 対象

- `nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/effect_return_summary_filter_tests.rs, nepl-core/tests/resource_ir.rs, stdlib/core/mem/pointer`

## 根拠

- 未記入

## 問題

Current main suppresses raw identity return summaries for direct RegionToken owner tokens. alloc_region returns Result<RegionToken<T>, str>, so RegionToken.raw allocation identity is not propagated to region_ptr, and checked MemPtr store/load/fill reports resource.raw.memory_outside_boundary even though the pointer is derived from a compiler-issued RegionToken.

## 影響

Safe public RegionToken -> MemPtr projections fail static checking and stdlib/core/mem/pointer doctests reject checked memory operations. This pressures callers toward raw helper bypasses and breaks the Stage 6 proof that checked MemPtr operations are accepted from source-proven RegionToken ownership.

## 修正方針

Separate public raw identity escape filtering from internal checked-MemPtr provenance summaries. Keep structural owner carriers such as builder/collection aggregates from becoming public raw identity carriers, but retain direct owner-token raw field provenance and Result<RegionToken, E> payload provenance for checked projection proof.

## 検証

Focused ResourceIR regression compile_accepts_checked_mem_ptr_wrapper_from_region_provenance, effect_return_summary_filter unit tests for direct owner token and aggregate carrier, and stdlib/core/mem/pointer focused doctests.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-18 Agent 1 解決内容

根本原因は、public raw identity escape を抑止するための owner-carrier filter が、checked `MemPtr` の内部 provenance summary まで同時に抑止していたことだった。`RegionToken<T>` は public surface では raw pointer escape として扱わないが、`region_ptr` / `region_ptr_at` から得た checked `MemPtr<T>` が store/load/fill を行うには、`RegionToken.raw` が allocator 由来であるという内部証跡が必要である。

修正後は、direct `RegionToken` と `Result<RegionToken, E>` payload の raw identity return summary を保持する。一方で、`StringBuilder` / `ByteBuilder` / collection storage のような structural owner carrier aggregate は引き続き raw identity summary carrier から外す。これにより Stage 6 の `MemPtr = non-owning pointer` / `RegionToken = free obligation owner` の分離を維持しつつ、checked wrapper の証明だけを復元した。

filter 名も `raw_identity_type_blocks_internal_summary` に合わせ、`str` と structural owner carrier aggregate は遮断し、compiler owner token 自体は遮断しない。public raw escape diagnostic は別経路の `raw_identity_return_projection_is_escape` が担うため、RegionToken を public raw pointer として漏らす設計には戻していない。

検証:

- `cargo fmt --all --check`
- `cargo test -p nepl-core --lib effect_return_summary_filter -- --nocapture`: 6 passed
- `cargo test -p nepl-core --test resource_ir compile_accepts_checked_mem_ptr_wrapper_from_region_provenance -- --nocapture`: passed
- `cargo check -p nepl-core --tests`
- `trunk build`
- `node nodesrc/tests.js -i stdlib\core\mem\pointer --no-tree -o tmp\agent1-core-mem-pointer-regiontoken-provenance.json -j 1 --dist web\dist --assert-io`: total=16, passed=16
