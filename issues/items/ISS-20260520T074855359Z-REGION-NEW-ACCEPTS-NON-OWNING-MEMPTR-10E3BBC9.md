---
id: ISS-20260520T074855359Z-REGION-NEW-ACCEPTS-NON-OWNING-MEMPTR-10E3BBC9
title: "region_new accepts non-owning MemPtr as owner-token input"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/core/mem/internal.nepl, stdlib/core/mem/pointer/region.nepl, nepl-core/src/resource/lower_raw_address.rs"
---

# ISS-20260520T074855359Z-REGION-NEW-ACCEPTS-NON-OWNING-MEMPTR-10E3BBC9: region_new accepts non-owning MemPtr as owner-token input

## 概要

region_new<T> currently takes MemPtr<T> and size to build RegionToken<T>. MemPtr is defined as a non-owning pointer view, so using it as the owner-token construction input keeps the API shape semantically inconsistent with the Stage 6 memory model even though Resource IR rejects known forged pointers.

## 対象

- `stdlib/core/mem/internal.nepl, stdlib/core/mem/pointer/region.nepl, nepl-core/src/resource/lower_raw_address.rs`

## 根拠

- `MemPtr<T>` は Stage 6 memory model で non-owning pointer view として固定する方針であり、free obligation owner の構築入力にしてはいけない。
- `region_new<T>` が `MemPtr<T>` を受け取る形のままだと、`region_ptr` / `region_ptr_at` / `str_addr` 由来の non-owning projection と allocator-issued owner identity の区別を API 形状で表現できない。
- Resource IR の dedicated lowering は `RegionToken.raw` を owner identity として扱うため、入力も `MemPtr.raw` field projection ではなく allocator / realloc が返した raw owner identity そのものに揃える必要がある。

## 問題

region_new<T> currently takes MemPtr<T> and size to build RegionToken<T>. MemPtr is defined as a non-owning pointer view, so using it as the owner-token construction input keeps the API shape semantically inconsistent with the Stage 6 memory model even though Resource IR rejects known forged pointers.

## 影響

The stdlib internal boundary still suggests that non-owning pointer views can be upgraded into free-obligation owners. This increases proof complexity around RegionToken provenance and makes self-host memory model design easier to copy incorrectly.

## 修正方針

Change region_new to take a raw owner identity i32 and size, update allocator/realloc callers to pass allocator-owned raw values directly, and update Resource IR dedicated lowering so RegionToken.raw aliases the first raw argument rather than a MemPtr.raw field. Keep forged raw values rejected by owner extent proof.

## 検証

Run focused core/mem Resource IR regressions for region_new forge rejection and allocated RegionToken deallocation, plus source policy and issue index checks.

## 2026-05-20 Agent 1 修正

`region_new<T>` の入力を `MemPtr<T>, i32` から `i32, i32` へ変更し、`MemPtr<T>` を owner-token construction input として使う形を廃止した。`RegionToken<T>` は引き続き過渡期の stdlib struct だが、internal boundary は allocator / realloc が返した raw owner identity と size を束ねるだけに限定し、`MemPtr<T>` は `region_ptr` / `region_ptr_at` から作る non-owning projection view に戻した。

compiler core 側では `MemoryHelperPrimitive::RegionNew` の Resource IR lowering を変更し、`RegionToken.raw` が第一引数 raw owner identity へ直接 alias するようにした。owner checker では raw alias transfer の source を `raw_owner_alias_transfer_source` で判定し、transferable owner がある raw identity、またはその storage origin が transferable owner を持つ場合だけ owner move を許可する。non-owning raw view はこの入口で拒否するため、borrowed projection を owner に昇格する経路は残さない。

owner summary 側では、returned raw owner leaf だけを raw owner seed にするようにした。`RegionToken.raw` のような owner leaf は caller 側へ伝播するが、`RegionToken.size` のような通常 metadata `i32` は owner として扱わない。これにより `region_new(raw, fake_size)` の extent mismatch は dealloc / realloc まで遅らず、summary 境界の `ConstructInput` で検出される。

関連ドキュメント:

- [static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [static_check_design_verification_20260430.md](../../doc/neplg2/static_check_design_verification_20260430.md)
- [stdlib_collection_mem_string_static_safety_design.md](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

検証:

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir region_token_forged -- --nocapture`: 6 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowers_dedicated_memory_helpers_once_per_call -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_new_extent_mismatch_through_summary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_new_extent_mismatch_before_realloc_summary -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_mem_ptr_alias_after_region_token -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_region_token_ptr_helper_alias_after_token_move -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_borrowed_region_ptr_at_known_offset_alias -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_rejects_borrowed_region_ptr_at_unknown_offset_dealloc_with_live_cell -- --nocapture`: passed
- `node nodesrc/test_stdlib_core_mem_boundary.js`: passed
- `node nodesrc/test_stdlib_mem_internal_region_new_docs.js`: passed
- `node nodesrc/issues.js check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-region-new-raw-memory-safety-after-trunk.json -j 1 --dist web/dist --assert-io`: 63 passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 25 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 30 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 39 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 40 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 42 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 43 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 58 --dist web/dist`: passed

補足:

- `node nodesrc/test_static_check_boundary_responsibility.js` は、この修正とは別件の `typecheck/driver.rs has 1701 lines; responsibility split limit is 1700` で失敗する。`typecheck/driver.rs` は今回の変更対象ではないため、次 issue として責務分割の根本対応を切り分ける。
- `cargo fmt -p nepl-core --check` は、この修正とは別件の `nepl-core/src/typecheck/prefix_check.rs` 既存整形差分で失敗する。該当 file には今回の内容差分がないため、この issue の commit には混ぜない。
