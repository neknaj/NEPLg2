---
id: ISS-20260517T012833283Z-SOURCECAPABILITIES-STILL-CARRIES-BRO-5588739B
title: "SourceCapabilities still carries broad file-level authority after exact use-site proof migration"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/proof.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T012833283Z-SOURCECAPABILITIES-STILL-CARRIES-BRO-5588739B: SourceCapabilities still carries broad file-level authority after exact use-site proof migration

## 概要

Exact use-site proof is now used by production gates, but SourceCapabilities still stores the old file-level SourceCapability set and exposes constructors that can authorize an entire file. This leaves a maintenance path back to file-scoped source capability authority.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/proof.rs, nodesrc/test_static_check_boundary_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `SourceCapabilities` が `SourceCapabilityUseSite` とは別に旧 `SourceCapability` enum と file-level `BTreeSet<SourceCapability>` を保持していた。
- `raw_memory_boundary()` / `with(SourceCapability)` / `insert(SourceCapability)` が残り、exact use-site proof を迂回して file 全体を許可する test helper や将来の production call path を作れていた。
- `nepl-core/tests/harness.rs` と `nepl-core/tests/resource_ir.rs` が entry source へ broad capability を手動注入しており、configured stdlib source と source evidence から証明する経路を回避していた。
- 旧 fallback を消すと、`core/mem/allocator.nepl` の raw helper 間呼び出しと `realloc` 由来 identity escape が source proof で表現できていないことも露出した。

## 問題

Exact use-site proof is now used by production gates, but SourceCapabilities still stores the old file-level SourceCapability set and exposes constructors that can authorize an entire file. This leaves a maintenance path back to file-scoped source capability authority.

## 影響

Future compiler or typecheck code can accidentally consume broad file-level authority instead of the exact proof event, weakening static-check precision and making proof-consumption bugs harder to catch.

## 修正方針

Remove the broad SourceCapability storage and constructors. Keep SourceCapabilities as a set of typed SourceCapabilityUseSite artifacts, and make any aggregate inspection helper derive from use-site evidence only. Update source policy to reject the old enum and broad constructors.

## 対応内容

- `SourceCapabilities` から旧 `SourceCapability` enum / file-level storage / broad constructor を削除し、`SourceCapabilityUseSite` の集合だけを保持する構造にした。
- test harness / ResourceIR fixture の手動 capability 注入を削除し、raw boundary fixture は configured stdlib root 配下の inline source path と source evidence で証明するようにした。
- `SourceCapabilityProof` の use-site builder と top-level raw helper propagation を責務分割し、直接 raw evidence を持つ helper だけを起点に top-level raw helper call-site へ証明を伝播するようにした。
- ResourceIR の raw identity escape 判定は、diagnostic span 内の typed raw operation proof を参照し、`Realloc` proof が内部 `Alloc` identity を返す境界を表すことを明示した。
- `nodesrc/test_static_check_boundary_responsibility.js` に旧 broad API 再導入禁止、source proof responsibility split、top-level raw helper proof propagation の監視を追加した。
- `doc/neplg2/static_check_complexity_reduction_plan.md` に Stage 6 の進捗として、file-level authority 廃止と use-site proof-only 化を追記した。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core source_map::tests -- --nocapture`
- `cargo test -p nepl-core raw_memory -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_raw_owner_through_str_from_addr -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_requires_struct_shape_for_compiler_memory_type_registration -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_allows_region_token_field_access_with_owner_field_boundary -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_rejects_mem_ptr_field_access_outside_compiler_memory_field_boundary -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir typecheck_allows_owner_backed_constructor_inside_compiler_owned_source -- --exact --nocapture`
