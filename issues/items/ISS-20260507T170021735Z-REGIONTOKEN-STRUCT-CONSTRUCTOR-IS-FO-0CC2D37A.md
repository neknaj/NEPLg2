---
id: ISS-20260507T170021735Z-REGIONTOKEN-STRUCT-CONSTRUCTOR-IS-FO-0CC2D37A
title: "RegionToken struct constructor is forgeable outside compiler memory boundary"
area: core
status: fixed
resolved: true
priority: P1
type: security
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T170021735Z-REGIONTOKEN-STRUCT-CONSTRUCTOR-IS-FO-0CC2D37A: RegionToken struct constructor is forgeable outside compiler memory boundary

## 概要

RegionToken is intended to represent the free-obligation owner side of the MemPtr/RegionToken split, but the ordinary struct constructor remains callable from user source. Resource IR owner checks reject several forged uses later, but the type layer still permits safe source to manufacture an owner-token-shaped value.

## 対象

- `nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `RegionToken<T>` は `free obligation owner` を表す token だが、通常の struct constructor として `RegionToken p size` を書ける状態だった。
- `str_addr` 由来の non-owning view を `RegionToken` へ包み直す direct constructor 経路は、Resource IR owner checker が後段で `resource.owner.no_free_obligation` として拒否できる一方、typecheck 済み HIR には owner-token-shaped value が残っていた。
- `stdlib/core/mem.nepl` の `region_new` は raw-memory-boundary capability を持つ compiler-owned boundary 内の constructor wrapper なので、ここは維持する必要がある。

## 問題

RegionToken is intended to represent the free-obligation owner side of the MemPtr/RegionToken split, but the ordinary struct constructor remains callable from user source. Resource IR owner checks reject several forged uses later, but the type layer still permits safe source to manufacture an owner-token-shaped value.

## 影響

Memory-safety enforcement depends on later Resource IR provenance recovery instead of making the owner-token construction boundary explicit. This keeps a forge route in the typed HIR surface and weakens the Stage 4 Resource IR owner model.

## 修正方針

Restrict direct RegionToken struct construction to files with raw-memory-boundary capability. User source must use compiler-owned stdlib allocation APIs such as region_new/alloc_region, whose Resource IR summaries preserve free-obligation provenance.

## 検証

Add focused Rust and doctest regressions for direct RegionToken construction, keep region_new non-owning-view rejection, run diagnostic registry/source-policy checks, and verify issue index.

## 修正結果

- `TypeDiagnosticCode::OwnerTokenConstructorRestricted` を追加し、stable code を `type.owner_token.constructor_restricted` とした。
- struct 定義時に `StructConstructorPolicy::{Public,RawMemoryBoundaryOnly}` を付与し、core memory boundary 内で定義された `RegionToken` だけを owner-token constructor として扱うようにした。
- constructor 適用時は policy を `match` し、`RawMemoryBoundaryOnly` の constructor が `raw_memory_boundary` capability を持たない source で呼ばれた場合に typecheck 段階で拒否する。
- 同名の user-defined `RegionToken` は `Public` policy のままなので、core owner token の制限が通常の user struct constructor へ波及しない。
- `raw_body_memory_operations_allowed` の内部判定を、より一般的な `raw_memory_boundary_allowed` helper に分離した。raw body effect gate は既存 helper 名のまま同じ capability 判定を使う。
- `region_new` 経由の non-owning view 昇格拒否は Resource IR owner checker に残し、direct constructor はそれ以前の type boundary で拒否する責務分割にした。

## 回帰テスト

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_region_token_struct_constructor_outside_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir typecheck_allows_user_struct_named_region_token -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view -- --nocapture`: passed
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-region-token-constructor-boundary.json -j 1 --dist web/dist`: total=18, passed=18
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
