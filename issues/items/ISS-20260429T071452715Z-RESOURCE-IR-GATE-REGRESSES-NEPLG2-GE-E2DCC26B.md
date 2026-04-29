---
id: ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B
title: "Resource IR gate regresses neplg2 generic aggregate and collection integration tests"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, nepl-core/tests/neplg2.rs, stdlib/alloc/collections"
---

# ISS-20260429T071452715Z-RESOURCE-IR-GATE-REGRESSES-NEPLG2-GE-E2DCC26B: Resource IR gate regresses neplg2 generic aggregate and collection integration tests

## 概要

cargo test -p nepl-core --test neplg2 -- --nocapture currently passes 52 tests and fails 8 tests. The failures are not caused by assignment diagnostic construction; they report resource.raw.ownership_violation as owner obligation leaks for generic aggregate store/load tests and RawMemoryLoadCell Uninit for List/HashMap collection paths.

## 対象

- `nepl-core/src/resource, nepl-core/tests/neplg2.rs, stdlib/alloc/collections`

## 根拠

- `cargo test -p nepl-core --test neplg2 -- --nocapture` は 60 件中 52 passed / 8 failed。
- generic aggregate 系は `generic_intrinsic_store_load_struct_preserves_fields`、`generic_store_uses_nested_address_call_without_stealing_value_arg`、`generic_hashkey_value_survives_hash_before_store`、`generic_hashkey_eq_after_load_uses_concrete_impl`、`generic_store_after_generic_trait_probe_preserves_struct` が、`Place { root: Local("p"), ... } still owns StorageId(0)` の owner obligation leak で失敗した。
- raw-memory-backed collection 系は `list_get_out_of_bounds_err`、`hashmap_custom_struct_key_roundtrips_value`、`llvm_hashmap_string_key_preserves_explicit_hasher_type_args` が、`List` / `HashMap` header や slot の `RawMemoryLoadCell ... found Uninit` で失敗した。
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md -i tests/compiler/neplg2.n.md --no-tree -o tmp/agent1-assignment-diagnostics-after-trunk.json -j 1` も 207 件中 206 passed / 1 failed で、失敗は `tests/compiler/neplg2.n.md::doctest#33` の `get__List_T_T_i32__Option_T_T__pure_i32` RawMemoryLoadCell Uninit だった。
- 同じ checkout で assignment diagnostic の focused regression `cargo test -p nepl-core --test neplg2 set_type_mismatch_is_error -- --nocapture` と `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 3 --dist web/dist` は pass しているため、assignment diagnostic code-first 化とは別の Resource IR 問題として扱う。

## 問題

cargo test -p nepl-core --test neplg2 -- --nocapture currently passes 52 tests and fails 8 tests. The failures are not caused by assignment diagnostic construction; they report resource.raw.ownership_violation as owner obligation leaks for generic aggregate store/load tests and RawMemoryLoadCell Uninit for List/HashMap collection paths.

## 影響

The full Rust neplg2 integration suite is no longer a clean regression gate. Generic aggregate storage and raw-memory-backed collections are directly relevant to self-host data structures, so this must be tracked without weakening RawMemoryLoadCell or owner obligation diagnostics.

## 修正方針

Trace Resource IR owner and cell-state propagation for generic aggregate parameters, raw collection headers, and slot loads. Preserve the strict diagnostics, but separate owned value obligations from initialized raw backing storage so valid generic store/load, List get, and HashMap roundtrip tests prove initialized cells instead of tripping D3100.

## 検証

cargo test -p nepl-core --test neplg2 generic_intrinsic_store_load_struct_preserves_fields generic_store_uses_nested_address_call_without_stealing_value_arg generic_hashkey_value_survives_hash_before_store generic_hashkey_eq_after_load_uses_concrete_impl generic_store_after_generic_trait_probe_preserves_struct list_get_out_of_bounds_err hashmap_custom_struct_key_roundtrips_value llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture; then cargo test -p nepl-core --test neplg2 -- --nocapture

## 2026-04-29 generic aggregate subcase 追記

現行 main で generic aggregate 系を再確認したところ、`roundtrip`、`same_after_store`、`hash_then_store`、`write_after_probe`、`write_nested` の test helper は `alloc_raw` で確保した一時 storage へ `store<T>` / `load<T>` した後、storage を `dealloc_raw` していなかった。

これは Resource IR owner checker の誤検出ではなく、Stage 4 の free obligation model が正しく検出した storage owner leak である。検査を弱めるのではなく、test helper 側で `load<T>` の戻り値を local に受け、raw storage を `dealloc_raw` してから戻り値を返すように修正した。

検証:

- `cargo test -p nepl-core --test neplg2 generic_intrinsic_store_load_struct_preserves_fields -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_hashkey_eq_after_load_uses_concrete_impl -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_hashkey_value_survives_hash_before_store -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_store_uses_nested_address_call_without_stealing_value_arg -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_ -- --nocapture`: 8 passed

collection 系の `List` / `HashMap` `RawMemoryLoadCell ... found Uninit` はこの test helper leak とは別の stdlib raw-memory-backed collection / Resource IR lowering 問題として残る。stdlib 側修正は別作業方針のため、この commit では generic aggregate subcase のみを整理する。
